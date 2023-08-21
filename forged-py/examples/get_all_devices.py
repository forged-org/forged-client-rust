#!/usr/bin/python3
import argparse
import asyncio
import gql
from forged import Forged


# TODO: This will be limited, and should technically iterate across the devices using pagination
id_query = gql.gql("""
    query GetDeviceIds {
        currentProvisioner {
            project {
                name
                devices {
                    nodes {
                        id
                        name
                    }
                }
            }
        }
    }""")


async def main():
    parser = argparse.ArgumentParser(description='Get all the IDs and names for devices in a project')
    parser.add_argument('--api', default='https://api.forged.dev', help='The API URL to communicate with')
    parser.add_argument('--token', default=None,
                        help='The provisioner token. FORGED_API_TOKEN will be probed from the '
                             'environment if not provided.')

    parser.add_argument('--num-devices', type=int, default=10, help='The number of devices to generate')
    parser.add_argument('--max-runs-per-device', type=int, default=3, help='The maximum number of device runs')

    args = parser.parse_args()
    async with Forged(args.token, url=args.api) as client:
        response = await client.session.execute(id_query)

    project = response["currentProvisioner"]["project"]
    devices = project["devices"]["nodes"]
    print(f'All devices for {project["name"]} ({len(devices)} in total)')
    for device in devices:
        print(f'{device["id"]}: {device["name"]}')

if __name__ == '__main__':
    asyncio.run(main())
