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
use tauri::Url;

pub struct Device {
    // ws_client: Arc<Client<WsClient>>,
    // vector of base64 images
    is_recording: bool,
    camera: Camera,
    captured_images: VecDeque<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    input_stream: Stream,
    output_stream: Stream,
    send_queue_tx: UnboundedSender<String>
}

struct WsClient {
    send_queue_tx: UnboundedSender<String>,
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

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        info!("received bytes: {bytes:?}");
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

        let (ws_client, _) =
            ezsockets::connect(|_client| WsClient { send_queue_tx }, config).await;

        tokio::spawn(message_sender(send_queue_rx, Arc::new(ws_client)));

        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let output_device = host.default_output_device().unwrap();

        println!("Using input device: \"{}\"", input_device.name()?);
        println!("Using output device: \"{}\"", output_device.name()?);

        let config: cpal::StreamConfig = input_device.default_input_config()?.into();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {

        };

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        };

        fn err_fn(err: cpal::StreamError) {
            error!("an error occurred on stream: {}", err);
        }

        let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
        let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None)?;

        // first camera in system
        let index = CameraIndex::Index(0);
        // request the absolute highest resolution CameraFormat that can be decoded to RGB.
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        // make the camera
        let mut camera = Camera::new(index, requested).unwrap();

        Ok(Self { captured_images: VecDeque::new(), send_queue_tx, camera, is_recording: false, input_stream, output_stream })
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
            self.input_stream.pause();
        }
        else {
            self.input_stream.play();
        }
        self.is_recording = !self.is_recording
    }

}

async fn message_sender(mut send_queue_rx: UnboundedReceiver<String>, ws_client: Arc<Client<WsClient>>){
    while let Some(message) =  send_queue_rx.recv().await {
        if let Err(signal) = ws_client.binary(message){
            error!("Error sending ws message to server. Details {}", signal);
        };
    }
}

fn put_kernel_messages_into_queue(mut send_queue_rx: UnboundedReceiver<String>){

}

fn record_audio(){
}

fn play_audiosegments() -> anyhow::Result<()>{

    Ok(())
}


// for client ? ... later
// fn run_device(server_url: String) -> anyhow::Result<()>{
//     let mut device = Device::create_with_server_url(server_url);
//     device.start()
// }

//** Device Logic to the best of my knowledge
//** ----------------------------------------

//**   1. Start
//**      - if 'TEACH_MODE' env variable is set to "True", don't do anything / quit

//**   2. make websocket connection to server_url
//**      - spawn task to receive data from [send_queue] and send it to server using websocket
//**      - log messages received from server
//**      - [accumulate] websocket message chunks
//**      - if message type if audio, convert and append audio bytes to [audiosegments]
//**      - if 'CODE_RUNNER' env variable is set to "True", check if websocket message is code to be executed by interpreter
//**        - send interpreter result to [send_queue]

//**   3. if 'CODE_RUNNER' env variable is set to "True" start [put_kernel_messages_into_queue] into [send_queue]
//**        - [[put_kernel_messages_into_queue]]
//**            - stream syslog, filter for open_interpreter messages and send to queue

//**   4. start thread to play audio ([play_audiosegments]) from [audiosegments]
//**      - play audio sequentially
//**      - on_release -

//**   5. start listening to keyboard for spacebar press/release
//**      - on_release -
//**        - if spacebar - [toggle_recording]
//**        - [[[toggle_recording]]]
//**            - some logic to toggle recording flag if
//**            - if recording is true. start thread to [record_audio]
//**            - [[[record_audio]]]
//**                - use env variables to determine if STT will run on server or on client
//**                - stream audio to server or transcribe on device
//**                - get all captured images, convert all to base64 png images, and add to each **sequentially** to [send_queue]
//**                - add audio bytes or transcribed text to [send_queue]
//**        - if CAMERA_ENABLED & 'c' key - [fetch_image_from_camera]()
//**        - [[[fetch_image_from_camera]]]
//**            - capture image from camera and append it to self.captured_images array
//**      - on_press
//**        - if spacebar - [toggle_recording]


//**     TLDR
//**     - fetch images from camera, store in array, and encode to base64
//**     - play audio
//**     - record audio
//**     - toggle recording
//**     - send message from queue to websocket
