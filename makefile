all:
	python3 sys_info.py --json | python3 mogAI.py | ./deploy_files.sh

.PHONY: all
