export type AppConfig = {
  theme_color: string;
  video_fit: 'cover' | 'contain';
  outputs: {
    audio: boolean;
    video: boolean;
    chat: boolean;
  };
  inputs: {
    mic: boolean;
    camera: boolean;
  };
};

export const useAppConfig = (): AppConfig => {
  return {
    theme_color: 'blue',
    video_fit: 'cover',
    outputs: {
      audio: true,
      video: true,
      chat: false,
    },
    inputs: {
      mic: true,
      camera: false,
    },
  };
};
