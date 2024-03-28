import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { DynamicIsland, IslandState, IslandStates } from "./components/DynamicIsland";
import { LiveKitRoom, RoomAudioRenderer, StartAudio, useToken } from "@livekit/components-react";
import Playground, { PlaygroundOutputs } from './components/Playground';
import { PlaygroundToast, ToastType } from './components/PlaygroundToast';
import { generateRandomAlphanumeric } from './utils/livekit';
import { AnimatePresence, motion } from "framer-motion";
import { CallNavBar } from './components/CallNavbar';
import { useAppConfig } from './hooks/useAppConfig';
import { WelcomePage } from "./components/Welcome";
import { tw } from "./utils/tw";

export default function App() {
  const [toastMessage, setToastMessage] = useState<{
    message: string;
    type: ToastType;
  } | null>(null);
  const [shouldConnect, setShouldConnect] = useState(false);
  const [roomName] = useState(createRoomName());
  const [liveKitUrl, setLiveKitUrl] = useState<string>();

  const [state, setState] = useState<IslandState>(IslandStates[0])
  const onboarding = false

  const tokenOptions = useMemo(() => {
    return {
      userInfo: { identity: generateRandomAlphanumeric(16) },
    };
  }, []);
  
  // const token = useToken('/api/get-participant-token', roomName, tokenOptions);
  const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MTE1ODUwNDEsImlzcyI6IkFQSUJ2NmdzQU5lclhaWCIsIm5iZiI6MTcxMTU4NDE0MSwic3ViIjoiY2hhZCIsInZpZGVvIjp7ImNhblB1Ymxpc2giOnRydWUsImNhblB1Ymxpc2hEYXRhIjp0cnVlLCJjYW5TdWJzY3JpYmUiOnRydWUsInJvb20iOiJjaGFkIiwicm9vbUpvaW4iOnRydWV9fQ.gpPZPvtzCn9Yh8JGhz-Yub98WyWQ_1KxyQAgkFVJRcA"
  
  const get_env=async(name: string) => {
    return await invoke("get_env", { name });
  }

  useEffect(()=>{
    get_env("NEXT_PUBLIC_LIVEKIT_URL").then((livekiturl) => {
      setLiveKitUrl(livekiturl as string)
      handleConnect(true)
    })
  },[])

  const appConfig = useAppConfig();

  const outputs = [
    appConfig?.outputs.audio && PlaygroundOutputs.Audio,
    appConfig?.outputs.video && PlaygroundOutputs.Video,
    appConfig?.outputs.chat && PlaygroundOutputs.Chat,
  ].filter((item) => typeof item !== 'boolean') as PlaygroundOutputs[];

  const handleConnect = useCallback((connect: boolean, opts?: { url: string; token: string }) => {
    if (connect && opts) {
      setLiveKitUrl(opts.url);
    }
    setShouldConnect(connect);
  }, []);
  
  if (onboarding){
    return <WelcomePage/>
  }

  return (
    <main className="h-dvh w-full flex flex-col items-center justify-center">
      <AnimatePresence>
        {toastMessage && (
          <motion.div
            className="left-0 right-0 top-0 absolute z-10"
            initial={{ opacity: 0, translateY: -50 }}
            animate={{ opacity: 1, translateY: 0 }}
            exit={{ opacity: 0, translateY: -50 }}
          >
            <PlaygroundToast
              message={
                toastMessage.message === 'Permission denied'
                  ? 'Please enable your microphone so i can hear you. 😊'
                  : toastMessage.message
              }
              type={toastMessage.type}
              onDismiss={() => setToastMessage(null)}
            />
          </motion.div>
        )}
      </AnimatePresence>

      <DynamicIsland state={state} />

      {liveKitUrl && (
        <LiveKitRoom
          className="flex flex-col h-full w-full"
          serverUrl={liveKitUrl}
          token={token}
          audio={appConfig.inputs.mic}
          video={false}
          connect={shouldConnect}
          onError={(e) => {
            setToastMessage({ message: e.message, type: 'error' });
            console.error(e);
          }}
        >
          <Playground
            outputs={outputs}
            themeColor={appConfig.theme_color}
            videoFit={appConfig.video_fit}
          />
          <RoomAudioRenderer />
          <StartAudio label="Click to enable audio playback" />
          <CallNavBar
            className="border-none bg-transparent [&>*:second-child]:bg-white [&>*:second-child]:rounded-full [&>*:second-child]:px-0 [&>*:second-child]:py-0 fixed bottom-6 mx-auto self-center"
          />
        </LiveKitRoom>
      )}
      
      <div className="flex space-x-3">
        {IslandStates.map((curr_state)=>(
          <button
              key={curr_state}
              className={tw("lowercase border ring-offset-1 px-3 py-2 rounded-3xl ring-1 ring-neutral-400",
                curr_state===state && "bg-black text-white"
              )}
              onClick={()=> setState(curr_state)}
          >
            {curr_state.replace("_", " ")}
          </button>
        ))}
      </div>
    </main>
  );
}

const createRoomName = () => {
  return [generateRandomAlphanumeric(4), generateRandomAlphanumeric(4)].join('-');
};