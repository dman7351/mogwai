#!/usr/bin/env python3
"""
System Hardware Information Reporter

This script gathers basic hardware information from Windows, macOS, and Linux systems
and generates a standardized report suitable for AI input or system analysis.
"""

import os
import platform
import json
import subprocess
import shutil
import sys
from datetime import datetime

try:
    import socket
except ImportError:
    pass


def get_size_format(bytes, factor=1024, suffix="B"):
    """
    Scale bytes to its proper format
    e.g:
        1253656 => '1.20 MB'
        1253656678 => '1.17 GB'
    """
    for unit in ["", "K", "M", "G", "T", "P"]:
        if bytes < factor:
            return f"{bytes:.2f} {unit}{suffix}"
        bytes /= factor
    return f"{bytes:.2f} Y{suffix}"


def get_windows_info():
    """Gather system information on Windows platforms using WMI."""
    try:
        import wmi
        import psutil
        
        info = {}
        
        # Create WMI connection
        c = wmi.WMI()
        
        # CPU Information
        processor_info = c.Win32_Processor()[0]
        info["cpu"] = {
            "model": processor_info.Name.strip(),
            "physical_cores": psutil.cpu_count(logical=False),
            "total_cores": psutil.cpu_count(logical=True),
            "max_frequency": f"{processor_info.MaxClockSpeed} MHz"
        }
        
        # Memory Information
        memory = psutil.virtual_memory()
        info["memory"] = {
            "total": get_size_format(memory.total),
            "available": get_size_format(memory.available),
            "used_percent": memory.percent
        }
        
        # Disk Information
        disks = []
        for disk in c.Win32_LogicalDisk(DriveType=3):  # Fixed disks only
            disks.append({
                "device": disk.DeviceID,
                "total": get_size_format(int(disk.Size or 0)),
                "free": get_size_format(int(disk.FreeSpace or 0)),
                "used_percent": (int(disk.Size or 1) - int(disk.FreeSpace or 0)) / int(disk.Size or 1) * 100 if disk.Size else 0
            })
        info["disks"] = disks
        
        # Network Information
        nics = []
        for nic in c.Win32_NetworkAdapterConfiguration(IPEnabled=True):
            adapter_info = {
                "name": nic.Description,
                "mac_address": nic.MACAddress,
                "ip_addresses": nic.IPAddress
            }
            nics.append(adapter_info)
        info["network"] = nics
        
    except ImportError:
        info = {"error": "WMI or psutil modules not available. Run 'pip install wmi psutil' to install."}
    
    return info


def get_linux_info():
    """Gather system information on Linux platforms."""
    try:
        import psutil
        info = {}
        
        # CPU Information
        info["cpu"] = {
            "physical_cores": psutil.cpu_count(logical=False),
            "total_cores": psutil.cpu_count(logical=True)
        }
        
        # Try to get CPU model name from /proc/cpuinfo
        try:
            with open("/proc/cpuinfo", "r") as f:
                for line in f:
                    if "model name" in line:
                        info["cpu"]["model"] = line.split(":")[1].strip()
                        break
        except:
            info["cpu"]["model"] = "Unknown"
        
        # Memory Information
        memory = psutil.virtual_memory()
        info["memory"] = {
            "total": get_size_format(memory.total),
            "available": get_size_format(memory.available),
            "used_percent": memory.percent
        }
        
        # Disk Information
        disks = []
        for partition in psutil.disk_partitions():
            if os.name == 'nt':
                if 'cdrom' in partition.opts or partition.fstype == '':
                    continue
            if os.name == 'posix':
                if partition.mountpoint == '/boot' or partition.mountpoint.startswith('/boot/'):
                    continue
            try:
                usage = psutil.disk_usage(partition.mountpoint)
                disks.append({
                    "device": partition.device,
                    "mountpoint": partition.mountpoint,
                    "filesystem": partition.fstype,
                    "total": get_size_format(usage.total),
                    "free": get_size_format(usage.free),
                    "used_percent": usage.percent
                })
            except Exception:
                pass
        info["disks"] = disks
        
        # Network Information
        nics = []
        for interface_name, interface_addresses in psutil.net_if_addrs().items():
            for address in interface_addresses:
                if address.family == psutil.AF_LINK:
                    mac = address.address
                    break
            else:
                mac = None
            
            nic_info = {
                "name": interface_name,
                "mac_address": mac,
                "ip_addresses": []
            }
            
            for address in interface_addresses:
                if address.family == socket.AF_INET or address.family == socket.AF_INET6:
                    nic_info["ip_addresses"].append(address.address)
            
            nics.append(nic_info)
        info["network"] = nics

    except ImportError:
        info = {"error": "psutil module not available. Run 'pip install psutil' to install."}
    
    return info


def get_macos_info():
    """Gather system information on macOS platforms."""
    try:
        import psutil
        import plistlib
        
        info = {}
        
        # Run system_profiler to get hardware info
        try:
            process = subprocess.Popen(['system_profiler', 'SPHardwareDataType', '-xml'], 
                                       stdout=subprocess.PIPE)
            output, _ = process.communicate()
            plist = plistlib.loads(output)
            
            hardware_info = plist[0]['_items'][0]
            
            # CPU Information
            info["cpu"] = {
                "model": hardware_info.get('cpu_type', 'Unknown') + ' ' + hardware_info.get('current_processor_speed', ''),
                "physical_cores": psutil.cpu_count(logical=False),
                "total_cores": psutil.cpu_count(logical=True)
            }
            
            # Memory Information
            memory = psutil.virtual_memory()
            info["memory"] = {
                "total": get_size_format(memory.total),
                "available": get_size_format(memory.available),
                "used_percent": memory.percent
            }
            
        except Exception as e:
            # Fallback to psutil if system_profiler fails
            info["cpu"] = {
                "physical_cores": psutil.cpu_count(logical=False),
                "total_cores": psutil.cpu_count(logical=True),
                "model": "Unknown (system_profiler failed)"
            }
            
            memory = psutil.virtual_memory()
            info["memory"] = {
                "total": get_size_format(memory.total),
                "available": get_size_format(memory.available),
                "used_percent": memory.percent
            }
        
        # Disk Information
        disks = []
        for partition in psutil.disk_partitions():
            try:
                usage = psutil.disk_usage(partition.mountpoint)
                disks.append({
                    "device": partition.device,
                    "mountpoint": partition.mountpoint,
                    "filesystem": partition.fstype,
                    "total": get_size_format(usage.total),
                    "free": get_size_format(usage.free),
                    "used_percent": usage.percent
                })
            except Exception:
                pass
        info["disks"] = disks
        
        # Network Information
        nics = []
        for interface_name, interface_addresses in psutil.net_if_addrs().items():
            for address in interface_addresses:
                if address.family == psutil.AF_LINK:
                    mac = address.address
                    break
            else:
                mac = None
            
            nic_info = {
                "name": interface_name,
                "mac_address": mac,
                "ip_addresses": []
            }
            
            for address in interface_addresses:
                if address.family == socket.AF_INET or address.family == socket.AF_INET6:
                    nic_info["ip_addresses"].append(address.address)
            
            nics.append(nic_info)
        info["network"] = nics
        
    except ImportError:
        info = {"error": "Required modules not available. Run 'pip install psutil' to install."}
    
    return info


def gather_system_info():
    """Gather all system information and return as a dictionary."""
    # Detect the operating system
    system = platform.system()
    
    # Common information across all platforms
    info = {
        "system": {
            "name": platform.system(),
            "version": platform.version(),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "hostname": platform.node()
        },
        "timestamp": datetime.now().isoformat()
    }
    
    # Get OS-specific information
    if system == "Windows":
        hw_info = get_windows_info()
    elif system == "Linux":
        hw_info = get_linux_info()
    elif system == "Darwin":  # macOS
        hw_info = get_macos_info()
    else:
        hw_info = {"error": f"Unsupported operating system: {system}"}
    
    # Combine information
    info.update(hw_info)
    
    # Check for installed dependencies - FIXED THIS PART
    info["dependencies"] = {}
    try:
        import psutil
        info["dependencies"]["psutil"] = True
    except ImportError:
        info["dependencies"]["psutil"] = False
        
    # Check for WMI only on Windows
    if system == "Windows":
        try:
            import wmi
            info["dependencies"]["wmi"] = True
        except ImportError:
            info["dependencies"]["wmi"] = False
    else:
        info["dependencies"]["wmi"] = False
    
    return info


def print_console_report(info):
    """Print a human-readable report to the console."""
    print("\n=== System Hardware Information Report ===")
    print(f"Operating System: {info['system']['name']} {info['system']['version']}")
    print(f"Machine Type: {info['system']['machine']}")
    print(f"Hostname: {info['system']['hostname']}")
    
    if "cpu" in info:
        print("\n--- CPU Information ---")
        for key, value in info["cpu"].items():
            print(f"{key.replace('_', ' ').title()}: {value}")
    
    if "memory" in info:
        print("\n--- Memory Information ---")
        for key, value in info["memory"].items():
            print(f"{key.replace('_', ' ').title()}: {value}")
    
    if "disks" in info:
        print("\n--- Disk Information ---")
        for i, disk in enumerate(info["disks"]):
            print(f"\nDisk {i+1}:")
            for key, value in disk.items():
                print(f"  {key.replace('_', ' ').title()}: {value}")
    
    if "network" in info:
        print("\n--- Network Information ---")
        for i, nic in enumerate(info["network"]):
            print(f"\nNetwork Interface {i+1}:")
            for key, value in nic.items():
                if key == "ip_addresses" and isinstance(value, list):
                    print(f"  IP Addresses: {', '.join(value) if value else 'None'}")
                else:
                    print(f"  {key.replace('_', ' ').title()}: {value}")


def main():
    """Main function to gather system information and output as requested."""
    try:
        # Gather all system information
        info = gather_system_info()
        
        # Check if we should output JSON
        json_output = len(sys.argv) > 1 and sys.argv[1] == "--json"
        
        if json_output:
            # Output ONLY JSON when the --json flag is used
            print(json.dumps(info, indent=2))
        else:
            # Print human-readable report
            print_console_report(info)
            
            # Save to JSON file
            filename = f"system_info_{platform.node()}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
            with open(filename, "w") as f:
                json.dump(info, f, indent=2)
            
            print(f"\nDetailed report saved to {filename}")
        
        return info
        
    except Exception as e:
        if len(sys.argv) > 1 and sys.argv[1] == "--json":
            sys.stderr.write(f"Error gathering system information: {e}\n")
        else:
            print(f"Error gathering system information: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()