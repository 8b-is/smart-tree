from fastapi import FastAPI

# Welcome to the main stage!
# This is where the magic happens for "The King's Code" API.
# Don't be cruel to this code, now.

app = FastAPI()

@app.get("/")
def read_root():
    """
    This is the root endpoint of our API.
    It's like the opening riff of a great song, letting you know you're in the right place.
    """
    return {"message": "Welcome to The King's Code API! Thank you, thank you very much."}

@app.get("/heartbeat")
def heartbeat():
    """
    A simple heartbeat endpoint to check if the API is alive and well.
    It's the steady rhythm that keeps the whole show going.
    """
    return {"status": "alive", "message": "I'm all shook up (in a good way)!"}