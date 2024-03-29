pub struct Device {
    // vector of base64 images
    pub captured_images: Vec<String>,
    // local or remote server url to make websocket connection to
    pub server_url: String
}

impl Device {
    fn new(){
        todo!()
    }
    
    fn create_with_server_url(server_url:String) -> Self {
        Device{
            server_url,
            captured_images: Vec::new(),
        } 
    }

    fn start(&mut self) -> anyhow::Result<()> {
        // if os.getenv('TEACH_MODE') != "True":
        todo!()
    }
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
