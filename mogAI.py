#!/usr/bin/env python3
"""
System Info to Mistral AI Adapter

This script takes system information in JSON format (from stdin or a file)
and sends it to a Mistral AI agent through their API.
"""

import base64
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
    # Load Kubernetes configuration for accessing secrets
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
    
    # Format the system information as a simplified summary for the AI
    system_summary = {
        "system": {
            "name": system_info.get("system", {}).get("name", "Unknown"),
            "version": system_info.get("system", {}).get("version", "Unknown"),
            "platform": system_info.get("system", {}).get("platform", "Unknown")
        }
    }
    
    # Add CPU information if available
    if "cpu" in system_info:
        system_summary["cpu"] = {
            "model": system_info["cpu"].get("model", "Unknown"),
            "physical_cores": system_info["cpu"].get("physical_cores", 0),
            "total_cores": system_info["cpu"].get("total_cores", 0)
        }
    
    # Add memory information if available
    if "memory" in system_info:
        # Extract numeric value from memory strings like "16.00 GB"
        memory_total = system_info["memory"].get("total", "0 MB")
        try:
            # Try to convert memory string to a numeric value in MB
            memory_parts = memory_total.split()
            if len(memory_parts) >= 2:
                value = float(memory_parts[0])
                unit = memory_parts[1].upper()
                
                # Convert to MB based on unit
                if "GB" in unit:
                    memory_mb = value * 1024
                elif "TB" in unit:
                    memory_mb = value * 1024 * 1024
                elif "KB" in unit:
                    memory_mb = value / 1024
                else:  # Assume MB or other
                    memory_mb = value
            else:
                memory_mb = 0
        except:
            memory_mb = 0
            
        system_summary["memory_mb"] = int(memory_mb)
    
    # Add disk information if available
    if "disks" in system_info and system_info["disks"]:
        # Sum up all disk sizes for total capacity
        total_disk_size = 0
        for disk in system_info["disks"]:
            disk_total = disk.get("total", "0 MB")
            try:
                # Try to convert disk string to a numeric value in MB
                disk_parts = disk_total.split()
                if len(disk_parts) >= 2:
                    value = float(disk_parts[0])
                    unit = disk_parts[1].upper()
                    
                    # Convert to MB based on unit
                    if "GB" in unit:
                        disk_mb = value * 1024
                    elif "TB" in unit:
                        disk_mb = value * 1024 * 1024
                    elif "KB" in unit:
                        disk_mb = value / 1024
                    else:  # Assume MB or other
                        disk_mb = value
                    
                    total_disk_size += disk_mb
                else:
                    continue
            except:
                continue
                
        system_summary["disk_size_mb"] = int(total_disk_size)
    
    # Define the conversation message with the system information
    messages = [
        {
            "role": "user",
            "content": json.dumps(system_summary, indent=2)
        }
    ]

    # Call the Mistral agent to get a response
    try:
        chat_response = client.agents.complete(
            agent_id=agent_id,
            messages=messages
        )

        print(chat_response.choices[0].message.content)
        
    except Exception as e:
        print(f"Error communicating with Mistral AI: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()