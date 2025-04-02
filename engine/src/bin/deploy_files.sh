#!/bin/bash
# deploy.sh
# This script extracts YAML file blocks from its input, writes each block to a file,
# and then optionally applies it using kubectl.
# After applying, it waits for the job to complete before processing the next block.
# Once a job completes, it creates a report and then deletes the job.

# Initialize variables
current_file=""
accumulator=""

# Read from STDIN (the entire output from the AI call)
while IFS= read -r line; do
    # Check for the start of a file block.
    if [[ "$line" =~ ^"=== FILE:" ]]; then
        # Extract the filename (e.g., "cpu_stress.rs" from "=== FILE: cpu_stress.rs ===")
        current_file=$(echo "$line" | sed -E 's/=== FILE: (.*) ===/\1/')
        accumulator=""  # Reset accumulator for new file.
    # Check for the end of the file block.
    elif [[ "$line" =~ ^"=== END FILE ===" ]]; then
        # Define output file name as <filename>.yaml
        output="${current_file}.yaml"
        # Write the accumulated YAML content to the file.
        echo "$accumulator" > "$output"
        echo "Created ${output}..."
        # Prompt the user whether to run the file, re-prompting if input is invalid.
        while true; do
            read -n 1 -p "Would you like to run ${output}? Y/N: " mainmenuinput < /dev/tty
            echo ""
            mainmenuinput=$(echo "$mainmenuinput" | tr '[:upper:]' '[:lower:]')
            if [[ "$mainmenuinput" == "y" || "$mainmenuinput" == "n" ]]; then
                break
            else
                echo "Unknown input: $mainmenuinput. Please enter Y or N."
            fi
        done
        if [ "$mainmenuinput" = "y" ]; then
            envsubst < "$output" > tempx.yaml
            kubectl apply -f tempx.yaml
            jobname="$current_file"
            echo "Waiting for job ${jobname} to complete..."
            kubectl wait --for=condition=complete --timeout=300s job/"$jobname"
            echo "Job ${jobname} completed."
            # Describe the job, save report, then delete the job.
            kubectl describe job "$jobname" > "report-${jobname}.txt"
            echo "Report for ${jobname} saved to report-${jobname}.txt."
            kubectl delete job "$jobname"
            echo "Job ${jobname} deleted."
        fi
        # Reset variables for the next file block.
        current_file=""
        accumulator=""
    else
        # If within a file block, accumulate the line.
        if [ -n "$current_file" ]; then
            accumulator+="$line"$'\n'
        fi
    fi
done
