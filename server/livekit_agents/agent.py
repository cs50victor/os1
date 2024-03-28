# Copyright 2023 LiveKit, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
    

import asyncio
from datetime import datetime
from enum import Enum
import inspect
import json
import logging
import os
from typing import AsyncIterable

from livekit import rtc, agents
from livekit.agents.tts import SynthesisEvent, SynthesisEventType
from supabase import Client, create_client
from llm import (
    ChatGPTMessage,
    ChatGPTMessageRole,
    ChatGPTPlugin,
)
from livekit.plugins.deepgram import STT
from livekit.plugins.elevenlabs import TTS, Voice, VoiceSettings

PROMPT = "You are Buildspace AI, a friendly voice assistant that connects buildspace members together. \
          Conversation should be personable, and be sure to ask follow up questions. \
          If your response is a question, please append a question mark symbol to the end of it.\
          Don't respond with more than a few sentences."

INTRO = "Hey! I'm Buildspace AI. Here to help you build your ideas, find other buildspace members you can connect with, and help you get discovered. So, what's your name and tell me a little bit about what you're building."

SIP_INTRO = "Hey! I'm Buildspace AI. Here to help you build your ideas, find other buildspace members you can connect with, and help you get discovered. So, what's your name and tell me a little bit about what you're building."


    
# convert intro response to a stream
async def intro_text_stream(sip: bool):
    if sip:
        yield SIP_INTRO
        return

    yield INTRO


AgentState = Enum("AgentState", "IDLE, LISTENING, THINKING, SPEAKING")

ELEVEN_TTS_SAMPLE_RATE = 24000
ELEVEN_TTS_CHANNELS = 1


class BuildspaceAI:
    @classmethod
    async def create(cls, ctx: agents.JobContext):
        buildspace = BuildspaceAI(ctx)
        await buildspace.start()

    def __init__(self, ctx: agents.JobContext):
        # plugins
        self.chatgpt_plugin = ChatGPTPlugin(
            prompt=PROMPT, message_capacity=20, model="gpt-4-1106-preview"
        )
        self.stt_plugin = STT(
            min_silence_duration=100,
        )
        self.tts_plugin = TTS(
            voice= Voice(
                id="iP95p4xoKVk53GoZ742B",
                name="Chris",
                category="premade",
                settings=VoiceSettings(
                    stability=0.71, similarity_boost=0.5, style=0.0, use_speaker_boost=True
                ),
            ),
            model_id="eleven_turbo_v2", 
            sample_rate=ELEVEN_TTS_SAMPLE_RATE
        )

        url: str = os.environ.get("SUPABASE_URL")
        key: str = os.environ.get("SUPABASE_SERVICE_KEY")
        self.supabase: Client = create_client(url, key)
        self.ctx: agents.JobContext = ctx
        self.chat = rtc.ChatManager(ctx.room)
        self.audio_out = rtc.AudioSource(ELEVEN_TTS_SAMPLE_RATE, ELEVEN_TTS_CHANNELS)

        self._sending_audio = False
        self._processing = False
        self._agent_state: AgentState = AgentState.IDLE

        self.chat.on("message_received", self.on_chat_received)
        self.ctx.room.on("track_subscribed", self.on_track_subscribed)

    async def start(self):
        # if you have to perform teardown cleanup, you can listen to the disconnected event
        # self.ctx.room.on("disconnected", your_cleanup_function)

        # publish audio track
        track = rtc.LocalAudioTrack.create_audio_track("agent-mic", self.audio_out)
        await self.ctx.room.local_participant.publish_track(track)

        # allow the participant to fully subscribe to the agent's audio track, so it doesn't miss
        # anything in the beginning
        await asyncio.sleep(1)

        sip = self.ctx.room.name.startswith("sip")
        await self.process_chatgpt_result(intro_text_stream(sip))
        self.update_state()

    def on_chat_received(self, message: rtc.ChatMessage):
        # TODO: handle deleted and updated messages in message context
        if message.deleted:
            return

        msg = ChatGPTMessage(role=ChatGPTMessageRole.user, content=message.message)
        chatgpt_result = self.chatgpt_plugin.add_message(msg)
        self.ctx.create_task(self.process_chatgpt_result(chatgpt_result))

    def on_track_subscribed(
        self,
        track: rtc.Track,
        publication: rtc.TrackPublication,
        participant: rtc.RemoteParticipant,
    ):
        self.ctx.create_task(self.process_track(track))

    async def process_track(self, track: rtc.Track):
        audio_stream = rtc.AudioStream(track)
        stream = self.stt_plugin.stream()
        self.ctx.create_task(self.process_stt_stream(stream))
        async for audio_frame_event in audio_stream:
            if self._agent_state != AgentState.LISTENING:
                continue
            stream.push_frame(audio_frame_event.frame)
        await stream.flush()

    def reprompt(self, data, msg:str) -> str:
        # handle later
        # tokenCount = 0
        contextText = "One of your main goals is to assist people with building their ideas and helping connect them with \
        other buildspace members. You currently only have information on participants from Buildspace \
        season 3. Given the following sections from previous \
        Buildspace seasons, answer the question using only that information when asked for recommendations or people to connect with, \
        or when asked to find or connect with people. If you are unsure and the answer \
        is not explicitly provided in the section below, say \
        'Sorry, I can't find anyone from Buildspace to connect you with.', then ask a clarifying question. Here's some context of closest matches from previous buildspace seasons: "
        
        data = data[1]    
        
        for match in data:
            print("data", data)
            print("match", match)
            contextText += f'Title of Demo Day Submission: {match["title"]}\
                Niche: {match["niche"]}\
                Summary: {match["description"]} \
                Full Description: {match["youtube_transcript"]} \
                Social: {match["social"]} \
                Buildspace Season: {match["season"]}'
        
        contextText+=f"\nuser's message: {msg}"
        
        return inspect.cleandoc(contextText);

    async def process_stt_stream(self, stream):
        buffered_text = ""
        async for event in stream:
            if event.alternatives[0].text == "":
                continue
            if event.is_final:
                buffered_text = " ".join([buffered_text, event.alternatives[0].text])

            if not event.end_of_speech:
                continue
            await self.ctx.room.local_participant.publish_data(
                json.dumps(
                    {
                        "text": buffered_text,
                        "timestamp": int(datetime.now().timestamp() * 1000),
                    }
                ),
                topic="transcription",
            )
            
            msg_embedding = await self.chatgpt_plugin.embed(buffered_text)
            if msg_embedding:
                data, count = self.supabase.rpc('match_person', {
                    "query_embedding": msg_embedding,
                    "match_threshold": 0.30,
                    "match_count": 4,
                }).execute()
                buffered_text = self.reprompt(data,buffered_text)
                

            msg = ChatGPTMessage(role=ChatGPTMessageRole.user, content=buffered_text)
            chatgpt_stream = self.chatgpt_plugin.add_message(msg)
            self.ctx.create_task(self.process_chatgpt_result(chatgpt_stream))
            buffered_text = ""

    async def process_chatgpt_result(self, text_stream):
        # ChatGPT is streamed, so we'll flip the state immediately
        self.update_state(processing=True)

        stream = self.tts_plugin.stream()
        # send audio to TTS in parallel
        self.ctx.create_task(self.send_audio_stream(stream))
        all_text = ""
        async for text in text_stream:
            stream.push_text(text)
            all_text += text

        self.update_state(processing=False)
        # buffer up the entire response from ChatGPT before sending a chat message
        await self.chat.send_message(all_text)
        await stream.flush()

    async def send_audio_stream(self, tts_stream: AsyncIterable[SynthesisEvent]):
        async for e in tts_stream:
            if e.type == SynthesisEventType.STARTED:
                self.update_state(sending_audio=True)
            elif e.type == SynthesisEventType.FINISHED:
                self.update_state(sending_audio=False)
            elif e.type == SynthesisEventType.AUDIO:
                await self.audio_out.capture_frame(e.audio.data)
        await tts_stream.aclose()

    def update_state(self, sending_audio: bool = None, processing: bool = None):
        if sending_audio is not None:
            self._sending_audio = sending_audio
        if processing is not None:
            self._processing = processing

        state = AgentState.LISTENING
        if self._sending_audio:
            state = AgentState.SPEAKING
        elif self._processing:
            state = AgentState.THINKING

        self._agent_state = state
        metadata = json.dumps(
            {
                "agent_state": state.name.lower(),
            }
        )
        self.ctx.create_task(self.ctx.room.local_participant.update_metadata(metadata))


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)

    async def job_request_cb(job_request: agents.JobRequest):
        logging.info("Accepting job for Buildspace AI")

        await job_request.accept(
            BuildspaceAI.create,
            identity="Buildspace AI",
            name="Buildspace AI",
            auto_subscribe=agents.AutoSubscribe.AUDIO_ONLY,
            auto_disconnect=agents.AutoDisconnect.DEFAULT,
        )

    worker = agents.Worker(request_handler=job_request_cb)
    agents.run_app(worker)
