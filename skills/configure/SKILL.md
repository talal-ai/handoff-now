---
name: configure
description: Locate and safely explain the handoff-now configuration and threshold/privacy options.
allowed-tools: Bash, Read
---

Run `handoff-now configure`, read the referenced JSON, and explain safe changes. Maintain `prepareAbovePercentage < handoffAbovePercentage < hardStopAbovePercentage`. Never place credentials in the file.

If the user requests OS credential storage, have them pipe the value through stdin to `handoff-now credential store`; never place the credential in a command argument or display it.
