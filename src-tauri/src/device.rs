// TODO: optimize file later, 

use std::{collections::VecDeque, str::FromStr, sync::Arc, time::Duration};
use ezsockets::{client::ClientCloseMode, Client, ClientConfig, CloseFrame, MessageStatus, RawMessage, SocketConfig, WSError};
use nokhwa::{pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat}, Camera};
use image::{EncodableLayout, ImageBuffer, ImageOutputFormat, Pixel, Rgba, RgbaImage};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use base64::{engine::general_purpose, Engine};
use serde_json::{json, Value};
use log::{error, info};
use axum::async_trait;
use ringbuf::HeapRb;
use tauri::Url;

pub struct Device {
    is_recording: bool,
    is_speaking: bool,
    camera: Camera,
    captured_images: VecDeque<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    audio_input_stream: Stream,
    audio_output_stream: Stream,
    send_queue_tx: UnboundedSender<String>
}

struct WsClient {
    send_queue_tx: UnboundedSender<String>,
    audio_producer: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>
}

#[async_trait]
impl ezsockets::ClientExt for WsClient {
    type Call = ();

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        let data: Value = serde_json::from_str(&text)?;
        let transcript = data["channel"]["alternatives"][0]["transcript"].clone();

        if transcript != Value::Null {
            if let Err(e) = self.send_queue_tx.send(transcript.to_string()) {
                error!("Error sending to LLM: {}", e);
            };
        }

        // if message type is code, run code using open interpreter
        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        info!("received bytes: {bytes:?}");
        // when you receive audio bytes
        // do some audio conversion to match sample width, frame_rate, and channels
        // send to output stream
        
        self.audio_producer.push().unwrap();

        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        info!("ON CALL: {call:?} from / to server");
        let () = call;
        Ok(())
    }
    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        info!("CONNECTED from / to server");
        Ok(())
    }

    async fn on_connect_fail(&mut self, e: WSError) -> Result<ClientCloseMode, ezsockets::Error> {
        error!("CONNECTION FAILED | {e} from / to server");
        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_close(
        &mut self,
        frame: Option<CloseFrame>,
    ) -> Result<ClientCloseMode, ezsockets::Error> {
        error!("CONNECTION CLOSED | {frame:?} from / to server");
        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_disconnect(&mut self) -> Result<ClientCloseMode, ezsockets::Error> {
        error!("disconnect from / to server");
        Ok(ClientCloseMode::Reconnect)
    }
}


impl Device {
    const AUDIO_FORMAT : usize  = 16;
    const AUDIO_CHANNELS : usize  = 1; //* MONO
    const AUDIO_SAMPLE_RATE : usize  = 44100; //* MONO

    // server_url : local or remote server url to make websocket connection to
    pub async fn new(server_url: String) -> anyhow::Result<Self> {
        let ws_url = server_url.replace("https", "wss");
        let config = ClientConfig::new(Url::from_str(&ws_url).unwrap())
            .socket_config(SocketConfig {
                heartbeat: Duration::from_secs(11),
                timeout: Duration::from_secs(30 * 60), // 30 minutes
                heartbeat_ping_msg_fn: Arc::new(|_t: Duration| {
                    // really important
                    RawMessage::Text(
                        json!({
                            "type": "KeepAlive",
                        })
                        .to_string(),
                    )
                }),
            })
            // .header("Authorization", &format!("Token {}", secure_token))
            ;

        let (send_queue_tx, send_queue_rx) = unbounded_channel::<String>();

        tokio::spawn(message_sender(send_queue_rx, Arc::new(ws_client)));

        // first camera in system
        let index = CameraIndex::Index(0);
        // request the absolute highest resolution CameraFormat that can be decoded to RGB.
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        // make the camera
        let mut camera = Camera::new(index, requested).unwrap();

        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let output_device = host.default_output_device().unwrap();

        println!("Using input device: \"{}\"", input_device.name()?);
        println!("Using output device: \"{}\"", output_device.name()?);

        let config: cpal::StreamConfig = input_device.default_input_config()?.into();

        fn err_fn(err: cpal::StreamError) {
            error!("an error occurred on stream: {}", err);
        }

        let audio_input_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // convert to wav format
            // transcribe locally or send to server
            //
        };

        let audio_input_stream = input_device.build_input_stream(&config, audio_input_callback, err_fn, None)?;

        let audio_output_latency_ms = 50.0_f32;
        let latency_frames = (audio_output_latency_ms / 1_000.0) * config.sample_rate.0 as f32;
        let latency_samples = latency_frames as usize * config.channels as usize;

        let mut audio_ring = HeapRb::<f32>::new(latency_samples * 1.5);
        let (mut audio_producer, mut audio_consumer) = audio_ring.split();

        let output_data_callback = move | data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // feed audio from eleven labs / websocket
            for sample in data {
                *sample = match audio_consumer.pop() {
                    Some(s) => s,
                    None => 0.0
                };
            }
        };

        let (ws_client, _) =
            ezsockets::connect(|_client| WsClient { send_queue_tx, audio_producer }, config).await;
        let audio_output_stream = output_device.build_output_stream(&config, output_data_callback, err_fn, None)?;

        audio_output_stream.play();

        Ok(Self { captured_images: VecDeque::new(), send_queue_tx, camera, is_recording: false, is_speaking: true, audio_input_stream, audio_output_stream })
    }

    pub fn fetch_image_from_camera(&mut self){
        // get a frame
        let frame = self.camera.frame().unwrap();
        // decode into an ImageBuffer
        let rgb_camera_image = frame.decode_image::<RgbFormat>().unwrap();

        self.captured_images.push_back(rgb_camera_image);
        info!("You now have {} images which will be sent along with your next audio message.", self.captured_images.len())
    }

    fn encode_image_to_base64(image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>) -> anyhow::Result<String> {
        let mut image_data: Vec<u8> = Vec::new();
        image_buffer.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::PNG)?;
        let res_base64 = general_purpose::STANDARD.encode(image_data);
        Ok(format!("data:image/png;base64,{}", res_base64))
    }

    fn add_image_to_send_queue(&mut self, image_buffer : ImageBuffer<Rgb<u8>, Vec<u8>>){
        let base64_image = encode_image_to_base64(image_buffer);
        let image_message = json!{
            "role": "user",
            "type": "image",
            "format": "base64.png",
            "content": base64_image
        };
        self.send_queue_tx.send(image_message)
    }

    fn queue_all_captured_images(&mut self){
        while Some(img) = self.captured_images.pop_front(){
            self.add_image_to_send_queue(img);
        }
    }

    pub fn toggle_recording(&mut self){
        if self.is_recording {
            self.audio_input_stream.pause();
            info!("Recording stopped.");
        }
        else {
            self.audio_input_stream.play();
            info!("Recording started... ")
        }
        self.is_recording = !self.is_recording
    }

    pub fn toggle_speaking(&mut self){
        if self.is_speaking {
            self.audio_output_stream.pause();
            info!("Speaking stopped.");
        }
        else {
            self.audio_output_stream.play();
            info!("Speaking started... ")
        }
        self.is_speaking = !self.is_speaking
    }

    pub fn send_message_to_server(&self) -> anyhow::Result<(), String>{
        self.send_queue_tx.send(message).map_err(|e| format!("Couldn't send message to OS1 server. Reason {e}"))
    }

}

async fn message_sender(mut send_queue_rx: UnboundedReceiver<String>, ws_client: Arc<Client<WsClient>>){
    while let Some(message) =  send_queue_rx.recv().await {
        if let Err(signal) = ws_client.binary(message){
            error!("Error sending ws message to server. Details {}", signal);
        };
    }
}

fn record_audio(){
}
