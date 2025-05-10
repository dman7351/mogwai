#!/usr/bin/env python3
"""
System Info to Mistral AI Adapter

This script takes raw system information text via piped stdin and sends it 
directly to a Mistral AI agent through their API without parsing or reformatting.

Usage:
    echo "some system info text" | ./mogAI.py
"""

import sys
import os


def load_env_file():
    try:
        if os.path.exists('.env'):
            with open('.env', 'r') as file:
                for line in file:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        key, value = line.split('=', 1)
                        os.environ[key.strip()] = value.strip().strip("'\"")
    except Exception:
        pass


def get_piped_input():
    if not sys.stdin.isatty():
        return sys.stdin.read().strip()
    print("Error: No piped input provided.")
    sys.exit(1)


def main():
    load_env_file()

    api_key = os.environ.get("MISTRAL_API_KEY")
    agent_id = os.environ.get("MISTRAL_AGENT_ID")
    if not api_key:
        print("Error: MISTRAL_API_KEY environment variable is not set")
        sys.exit(1)
    if not agent_id:
        print("Error: MISTRAL_AGENT_ID environment variable is not set")
        sys.exit(1)

    raw_text = get_piped_input()
    if not raw_text:
        print("Error: Empty input received.")
        sys.exit(1)

    from mistralai import Mistral
    client = Mistral(api_key=api_key)

    messages = [{"role": "user", "content": raw_text}]

    try:
        resp = client.agents.complete(agent_id=agent_id, messages=messages)
        print(resp.choices[0].message.content)
    except Exception as e:
        print(f"Error communicating with Mistral AI: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
