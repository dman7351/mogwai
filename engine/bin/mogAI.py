#!/usr/bin/env python3
"""
System Info to Mistral AI Adapter

This script takes system information in JSON format (from stdin or a file)
and sends it to a Mistral AI agent through their API without reformatting.
"""

import json
import sys
import os

# Add this to load environment variables from .env file
def load_env_file():
    try:
        if os.path.exists('.env'):
            with open('.env', 'r') as file:
                for line in file:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        key, value = line.split('=', 1)
                        os.environ[key.strip()] = value.strip().strip('"\'')
            return True
        return False
    except Exception as e:
        print(f"Error loading .env file: {e}")
        return False

# Load environment variables from .env
load_env_file()

from mistralai import Mistral

def main():
    """Main function to process system information and send to Mistral AI."""
    # Load environment variables for API access
    try:
        api_key = os.environ.get("MISTRAL_API_KEY")
        agent_id = os.environ.get("MISTRAL_AGENT_ID")
        
        # Exit with an informative message if API key or agent ID is missing
        if not api_key:
            print("Error: MISTRAL_API_KEY environment variable is not set")
            sys.exit(1)
        if not agent_id:
            print("Error: MISTRAL_AGENT_ID environment variable is not set")
            sys.exit(1)
            
    except Exception as e:
        print(f"Error accessing environment variables: {e}")
        sys.exit(1)

    # Initialize Mistral client with the API key
    client = Mistral(api_key=api_key)

    # Read system information from stdin or a file
    try:
        # Check if we have input from a pipe
        if not sys.stdin.isatty():
            data = sys.stdin.read()
            if not data:
                print("Error: No data received from stdin")
                sys.exit(1)
            try:
                system_info = json.loads(data)
            except json.JSONDecodeError as e:
                print(f"Error: Invalid JSON data received: {e}")
                print(f"Data received: {data[:100]}...")  # Print first 100 chars for debugging
                sys.exit(1)
        # Otherwise, check if a file was specified
        elif len(sys.argv) > 1:
            try:
                with open(sys.argv[1], 'r') as f:
                    system_info = json.load(f)
            except FileNotFoundError:
                print(f"Error: File not found: {sys.argv[1]}")
                sys.exit(1)
            except json.JSONDecodeError as e:
                print(f"Error: Invalid JSON in file {sys.argv[1]}: {e}")
                sys.exit(1)
        else:
            print("Error: No input provided. Either pipe JSON data or specify a JSON file.")
            sys.exit(1)
            
    except Exception as e:
        print(f"Error reading input: {e}")
        sys.exit(1)
    
    # Define the conversation message with the raw system information
    messages = [
        {
            "role": "user",
            "content": json.dumps(system_info)
        }
    ]

    # Call the Mistral agent to get a response
    try:
        chat_response = client.agents.complete(
            agent_id=agent_id,
            messages=messages
        )
        print(chat_response.choices[0].message.content)
        
        # print(messages[0]["content"])
        
    except Exception as e:
        print(f"Error communicating with Mistral AI: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()