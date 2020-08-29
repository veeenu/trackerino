import numpy as np
import time
import asyncio
import aiohttp
from tqdm import tqdm

# this is not really the best possible way of timing a web resource btw


async def timed_fetch(session, uri):
    start = time.time()
    response = await session.get(uri)
    end = time.time()
    return end - start


async def main():
    uris = [
        "http://127.0.0.1:9000/tracking?ori=http://google.it&ref=https://twitch.tv/johndisandonato&path=/prova",
    ]

    num_requests = 1999

    async with aiohttp.ClientSession() as session:
        times = []
        for uri in uris:
            for i in tqdm(range(num_requests)):
                try:
                    duration = await timed_fetch(session, uri)
                    times.append(duration)
                except:
                    print(f'Couldn\'t connect: {uri}')
        times = np.array(times)
        print(
            f'\n{uri[:15]:>15s} > mean {np.mean(times)} std {np.std(times)} samples {len(times)}')

if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
