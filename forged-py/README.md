# Forged Python Client

This package is a client to interface with the forged.dev manufacturing/provisioning service.

When used in the automated Forged provisioner UI, uploading data blocks can be accomplished as
simply as:
```py
from forged import Forged

async def main():
    await Forged.upload_value("my_block", 10.0)
    blocks = await Forged.blocks()
    assert blocks["my_block"] == 10.0
```
