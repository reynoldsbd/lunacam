# Deployment helpers

export device ?= 192.168.7.3
DEPLOY_DEVICE = $(device)

DEPLOY_DEST = ~/lcdeploy/$(DEPLOY_COMPONENT)

DEPLOY_INIT = ssh $(DEPLOY_DEVICE) mkdir -p $(DEPLOY_DEST)

DEPLOY_FILES = scp -r $(1) $(DEPLOY_DEVICE):$(DEPLOY_DEST)

DEPLOY_CMD = ssh $(DEPLOY_DEVICE) "cd $(DEPLOY_DEST) && $(2) sudo -E $(1)"
