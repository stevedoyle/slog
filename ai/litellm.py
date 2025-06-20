# A simple example of how to send prompts to an LLM using LiteLLM:

#
# Set your API key as shown in the Google Colab
#
import os
from google.colab import userdata

api_key = userdata.get("OPENAI_API_KEY")
os.environ["OPENAI_API_KEY"] = api_key

from litellm import completion
from typing import List, Dict


def generate_response(messages: List[Dict]) -> str:
    """Call LLM to get response"""
    response = completion(model="openai/gpt-4o", messages=messages, max_tokens=1024)
    return response.choices[0].message.content


messages = [
    {
        "role": "system",
        "content": "You are an expert software engineer that prefers functional programming.",
    },
    {
        "role": "user",
        "content": "Write a function to swap the keys and values in a dictionary.",
    },
]

response = generate_response(messages)
print(response)
