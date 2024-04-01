from source.server.utils.local_mode import select_local_model
from source.server.tunnel import create_tunnel
from source.server.server import main
import asyncio
import signal
import typer
import os

app = typer.Typer()

@app.command()
def run():
    try:
        asyncio.run(main("0.0.0.0",10001,"litellm","gpt-4",False,False,2048,4096,0.8,"openai","openai"))
    except KeyboardInterrupt:
        os.kill(os.getpid(), signal.SIGINT)
