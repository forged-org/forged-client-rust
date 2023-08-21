#!/usr/bin/python3
"""
Description: Provides an example to query all of the forged devices in a project.
"""
import argparse
import asyncio
import gql
from forged import Forged

# The number of devices to query each time
PAGE_SIZE = 200

# The raw GraphQL query.
id_query = gql.gql("""
    query GetDeviceIds ($cursor: String, $first: Int!) {
        currentProvisioner {
            project {
                name
                devices (after: $cursor, first: $first) {
                    edges {
                        cursor
                    }
                    nodes {
                        id
                        name
                    }
                    pageInfo {
                        hasNextPage
                    }
                }
            }
        }
    }""")


async def fetch_all(client):
    """ Fetch all of the device IDs from a project. """
    devices = []
    result = await client.session.execute(id_query, variable_values={'first': PAGE_SIZE})
    result_devices = result["currentProvisioner"]["project"]["devices"]
    project_name = result["currentProvisioner"]["project"]["name"]
    devices += result_devices["nodes"]

    # Loop over the Relay connection and collect all of the pages of devices.
    while result_devices["pageInfo"]["hasNextPage"]:
        result = await client.session.execute(id_query,
            variable_values={
                'cursor': result_devices["edges"][-1]["cursor"],
                'first': PAGE_SIZE
            })

        # Add the current page to the list.
        result_devices = result["currentProvisioner"]["project"]["devices"]
        devices += result_devices["nodes"]

    # Return the devices and project name.
    return devices, project_name


async def main():
    parser = argparse.ArgumentParser(description='Get all the IDs and names for devices in a project')
    parser.add_argument('--api', default='https://api.forged.dev', help='The API URL to communicate with')
    parser.add_argument('--token', default=None,
                        help='The provisioner token. FORGED_API_TOKEN will be probed from the '
                             'environment if not provided.')

    args = parser.parse_args()
    async with Forged(args.token, url=args.api) as client:
        devices, project_name = await fetch_all(client)

    print(f'All devices for {project_name} ({len(devices)} in total)')
    for device in devices:
        print(f'{device["id"]}: {device["name"]}')

if __name__ == '__main__':
    asyncio.run(main())
