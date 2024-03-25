from dotenv import load_dotenv
load_dotenv()  # take environment variables from .env.

import asyncio
import subprocess
import platform

from .logs import setup_logging
from .logs import logger
setup_logging()

def get_kernel_messages():
    """
    Is this the way to do this?
    """
    current_platform = platform.system()
    
    if current_platform == "Darwin":
        process = subprocess.Popen(['syslog'], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        output, _ = process.communicate()
        return output.decode('utf-8')
    elif current_platform == "Linux":
        with open('/var/log/dmesg', 'r') as file:
            return file.read()
    else:
        logger.info("Unsupported platform.")

def custom_filter(message):
    # Check for {TO_INTERPRETER{ message here }TO_INTERPRETER} pattern
    if '{TO_INTERPRETER{' in message and '}TO_INTERPRETER}' in message:
        start = message.find('{TO_INTERPRETER{') + len('{TO_INTERPRETER{')
        end = message.find('}TO_INTERPRETER}', start)
        return message[start:end]
    # Check for USB mention
    # elif 'USB' in message:
    #     return message
    # # Check for network related keywords
    # elif any(keyword in message for keyword in ['network', 'IP', 'internet', 'LAN', 'WAN', 'router', 'switch']) and "networkStatusForFlags" not in message:
        
    #     return message
    else:
        return None
    
last_messages = ""

def check_filtered_kernel():
    messages = get_kernel_messages()
    messages.replace(last_messages, "")
    messages = messages.split("\n")
    
    filtered_messages = []
    for message in messages:
        if custom_filter(message):
            filtered_messages.append(message)
    
    return "\n".join(filtered_messages)

async def put_kernel_messages_into_queue(queue):
    while True:
        text = check_filtered_kernel()
        if text:
            if isinstance(queue, asyncio.Queue):
                await queue.put({"role": "computer", "type": "console", "start": True})
                await queue.put({"role": "computer", "type": "console", "format": "output", "content": text})
                await queue.put({"role": "computer", "type": "console", "end": True})
            else:
                queue.put({"role": "computer", "type": "console", "start": True})
                queue.put({"role": "computer", "type": "console", "format": "output", "content": text})
                queue.put({"role": "computer", "type": "console", "end": True})
        
        await asyncio.sleep(5)