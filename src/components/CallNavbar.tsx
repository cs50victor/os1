import { PlaygroundDeviceSelector } from './PlaygroundDeviceSelector';
import { TrackToggle } from '@livekit/components-react';
import { Menubar, MenubarMenu } from './ui/menubar';
import {  Track } from 'livekit-client';

export const CallNavBar = ({
  className,
}: {
  className?: string;
}) => {
  return (
    <Menubar className={className}>
      <MenubarMenu>
        <TrackToggle
          className="p-2 bg-gray-900 text-gray-300 border border-gray-800 rounded-sm hover:bg-gray-800"
          source={Track.Source.Microphone}
        />
      </MenubarMenu>
      <MenubarMenu>
        <PlaygroundDeviceSelector kind="audioinput" />
      </MenubarMenu>
    </Menubar>
  );
};
