import { LoadingSVG } from './LoadingSVG';
import { NameValueRow } from './NameValueRow';
import { PlaygroundTile } from './PlaygroundTile';
import { AgentMultibandAudioVisualizer } from './AgentMultibandAudioVisualizer';
import { useMultibandTrackVolume } from '../hooks/useTrackVolume';
import { AgentState } from '../utils/types';
import {
  TrackReference,
  VideoTrack,
  useChat,
  useConnectionState,
  useDataChannel,
  useLocalParticipant,
  useRemoteParticipants,
  useTracks,
} from '@livekit/components-react';
import { ConnectionState, LocalParticipant, RoomEvent, Track } from 'livekit-client';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Separator } from './ui/separator';
import { ChatMessageType, ChatTile } from './ChatTile';

export enum PlaygroundOutputs {
  Video,
  Audio,
  Chat,
}

export interface PlaygroundProps {
  themeColor: string;
  outputs?: PlaygroundOutputs[];
  videoFit?: 'contain' | 'cover';
}

const headerHeight = 56;

export default function Playground({ outputs, themeColor, videoFit }: PlaygroundProps) {
  const [agentState, setAgentState] = useState<AgentState>('offline');
  const [transcripts, setTranscripts] = useState<any[]>([]);
  const { send: sendChat, chatMessages } = useChat();
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  const { localParticipant } = useLocalParticipant();

  const participants = useRemoteParticipants({
    updateOnlyOn: [RoomEvent.ParticipantMetadataChanged],
  });
  const agentParticipant = participants.find((p) => p.isAgent);

  const visualizerState = useMemo(() => {
    if (agentState === 'thinking') {
      return 'thinking';
    } else if (agentState === 'speaking') {
      return 'talking';
    }
    return 'idle';
  }, [agentState]);

  const roomState = useConnectionState();
  const tracks = useTracks();

  const agentAudioTrack = tracks.find(
    (trackRef) => trackRef.publication.kind === Track.Kind.Audio && trackRef.participant.isAgent,
  );

  const agentVideoTrack = tracks.find(
    (trackRef) => trackRef.publication.kind === Track.Kind.Video && trackRef.participant.isAgent,
  );

  const subscribedVolumes = useMultibandTrackVolume(agentAudioTrack?.publication.track, 5);

  const localTracks = tracks.filter(({ participant }) => participant instanceof LocalParticipant);
  const localVideoTrack = localTracks.find(({ source }) => source === Track.Source.Camera);
  const localMicTrack = localTracks.find(({ source }) => source === Track.Source.Microphone);

  const localMultibandVolume = useMultibandTrackVolume(localMicTrack?.publication.track, 20);

  useEffect(() => {
    const allMessages = [...transcripts];
    for (const msg of chatMessages) {
      const isAgent = msg.from?.identity === agentParticipant?.identity;
      const isSelf = msg.from?.identity === localParticipant?.identity;
      let name = msg.from?.name;
      if (!name) {
        if (isAgent) {
          name = "Agent";
        } else if (isSelf) {
          name = "You";
        } else {
          name = "Unknown";
        }
      }
      allMessages.push({
        name,
        message: msg.message,
        timestamp: msg?.timestamp,
        isSelf: isSelf,
      });
    }
    allMessages.sort((a, b) => a.timestamp - b.timestamp);
    setMessages(allMessages);
  }, [transcripts, chatMessages, localParticipant, agentParticipant]);

  const isAgentConnected = agentState !== 'offline';

  const onDataReceived = useCallback(
    (msg: any) => {
      if (msg.topic === 'transcription') {
        const decoded = JSON.parse(new TextDecoder('utf-8').decode(msg.payload));
        let timestamp = new Date().getTime();
        if ('timestamp' in decoded && decoded.timestamp > 0) {
          timestamp = decoded.timestamp;
        }
        setTranscripts([
          ...transcripts,
          {
            name: 'You',
            message: decoded.text,
            timestamp: timestamp,
            isSelf: true,
          },
        ]);
      }
    },
    [transcripts],
  );

  useDataChannel(onDataReceived);

  const chatTileContent = useMemo(() => {
    return (
      <ChatTile
        messages={messages}
        accentColor={themeColor}
        onSend={sendChat}
      />
    );
  }, [messages, themeColor, sendChat]);

  const mixedMediaContent = useMemo(() => {
    return (
      <MixedMedia
        {...{
          agentVideoTrack,
          agentAudioTrack,
          agentState,
          videoFit,
          subscribedVolumes,
          themeColor,
        }}
      />
    );
  }, [agentAudioTrack, subscribedVolumes, themeColor, agentState, agentVideoTrack, videoFit]);

  return (
    <>
      <div
        className={`flex gap-4 py-4 grow w-full selection:bg-${themeColor}-900`}
        style={{ height: `calc(100% - ${headerHeight}px)` }}
      >
        <PlaygroundTile
          className="w-full h-full grow"
          childrenClassName="justify-center"
          status={
            <div className="ml-4 flex items-center justify-center space-x-3 text-inherit">
              <Separator className="h-3 text-gray-500" orientation="vertical" />
              <div className="flex space-x-3">
                <NameValueRow
                  name="Room connected"
                  value={
                    roomState === ConnectionState.Connecting ? (
                      <LoadingSVG diameter={16} strokeWidth={2} />
                    ) : (
                      roomState
                    )
                  }
                  valueColor={
                    roomState === ConnectionState.Connected ? `${themeColor}-500` : 'gray-500'
                  }
                />
                <Separator className="h-3 text-gray-500" orientation="vertical" />
                <NameValueRow
                  name="Agent connected"
                  value={
                    isAgentConnected ? (
                      'true'
                    ) : roomState === ConnectionState.Connected ? (
                      <LoadingSVG diameter={12} strokeWidth={2} />
                    ) : (
                      'false'
                    )
                  }
                  valueColor={isAgentConnected ? `${themeColor}-500` : 'gray-500'}
                />
                <Separator className="h-3 text-gray-500" orientation="vertical" />
                <NameValueRow
                  name="Agent status"
                  value={
                    agentState !== 'offline' && agentState !== 'speaking' ? (
                      <div className="flex gap-2 items-center">
                        <LoadingSVG diameter={12} strokeWidth={2} />
                        {agentState}
                      </div>
                    ) : (
                      agentState
                    )
                  }
                  valueColor={agentState === 'speaking' ? `${themeColor}-500` : 'gray-500'}
                />
              </div>
            </div>
          }
        >
          {mixedMediaContent}
        </PlaygroundTile>
        <PlaygroundTile
            className="h-full grow basis-1/4 hidden lg:flex"
          >
            {chatTileContent}
          </PlaygroundTile>
      </div>
    </>
  );
}

const MixedMedia = ({
  agentVideoTrack,
  agentAudioTrack,
  agentState,
  videoFit,
  subscribedVolumes,
  themeColor,
}: {
  themeColor: string;
  agentVideoTrack?: TrackReference;
  subscribedVolumes: Float32Array[];
  agentAudioTrack?: TrackReference;
  agentState: AgentState;
  videoFit: PlaygroundProps['videoFit'];
}) => {
  if (agentVideoTrack) {
    const videoFitClassName = `object-${videoFit}`;
    return (
      <div className="flex flex-col w-full grow text-gray-950 bg-black rounded-sm relative">
        <VideoTrack
          trackRef={agentVideoTrack}
          className={`absolute top-1/2 -translate-y-1/2 ${videoFitClassName} object-position-center w-full h-full`}
        />
      </div>
    );
  } else if (agentAudioTrack) {
    return (
      <div className="flex items-center justify-center w-full">
        <AgentMultibandAudioVisualizer
          state={agentState}
          barWidth={30}
          minBarHeight={30}
          maxBarHeight={150}
          accentColor={themeColor}
          accentShade={500}
          frequencies={subscribedVolumes}
          borderRadius={12}
          gap={16}
        />
      </div>
    );
  }
  return (
    <div className="flex items-center justify-center w-full">
      <div className="flex flex-col items-center gap-2 text-gray-700 text-center w-full">
        <LoadingSVG />
        waiting for audio / video from buildspace AI
      </div>
    </div>
  );
};
