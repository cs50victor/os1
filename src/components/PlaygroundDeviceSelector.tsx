import { useMediaDeviceSelect } from '@livekit/components-react';
import { useEffect, useState } from 'react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

type PlaygroundDeviceSelectorProps = {
  kind: MediaDeviceKind;
};

export const PlaygroundDeviceSelector = ({ kind }: PlaygroundDeviceSelectorProps) => {
  const [showMenu, setShowMenu] = useState(false);
  const deviceSelect = useMediaDeviceSelect({ kind: kind });
  const [selectedDeviceName, setSelectedDeviceName] = useState('');

  useEffect(() => {
    deviceSelect?.devices?.forEach((device) => {
      if (device.deviceId === deviceSelect.activeDeviceId) {
        setSelectedDeviceName(device.label);
      }
    });
  }, [deviceSelect.activeDeviceId, deviceSelect.devices, selectedDeviceName]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (showMenu) {
        setShowMenu(false);
      }
    };
    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  }, [showMenu]);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          className="flex gap-2 items-center px-2 py-1 bg-gray-900 text-gray-300 border border-gray-800 rounded-sm hover:bg-gray-800"
          onClick={(e) => {
            setShowMenu(!showMenu);
            e.stopPropagation();
          }}
        >
          <span className="max-w-[80px] overflow-ellipsis overflow-hidden whitespace-nowrap">
            {selectedDeviceName}
          </span>
          <ChevronSVG />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56 border border-gray-800 rounded-lg z-10 bg-inherit text-inherit">
        <DropdownMenuLabel>Audio Device</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {deviceSelect?.devices?.map((device, index) => (
          <DropdownMenuItem
            onClick={() => {
              deviceSelect.setActiveMediaDevice(device.deviceId);
              setShowMenu(false);
            }}
            className={`${
              device.deviceId === deviceSelect.activeDeviceId
                ? 'text-white font-semibold'
                : 'text-gray-500'
            }`}
            key={index}
          >
            {device.label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const ChevronSVG = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" fill="none">
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M3 5H5V7H3V5ZM7 9V7H5V9H7ZM9 9V11H7V9H9ZM11 7V9H9V7H11ZM11 7V5H13V7H11Z"
      fill="currentColor"
      fillOpacity="0.8"
    />
  </svg>
);
