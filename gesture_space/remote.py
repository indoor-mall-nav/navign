import requests
from openai import OpenAI
from config import OPENAI_KEY, DEEPSEEK_KEY
from shared import prompt


def is_chatgpt_api_available():
    # Step 1: Geolocate the IP address
    geo_response = requests.get("https://api.ip.sb/geoip")
    if geo_response.status_code != 200:
        return True
    country = geo_response.json().get("country_name")

    # Step 2: Check against OpenAI's supported regions
    unsupported_countries = ["China", "Hong Kong", "Macau", "Russia", "Belarus", "Iran", "Syria", "North Korea", "Cuba"]
    return country not in unsupported_countries


def get_appropriate_client() -> tuple[OpenAI, str]:
    if is_chatgpt_api_available():
        return OpenAI(api_key=OPENAI_KEY), "gpt-4o"
    else:
        return OpenAI(
            api_key=DEEPSEEK_KEY, base_url="https://api.deepseek.com"
        ), "deepseek-chat"


def run_remote_response(env: str, user: str) -> str:
    client, model = get_appropriate_client()

    content = 'The environment involves following surroundings:\n' + env + \
        f'\nThe user is asking: {user}\n'

    response = client.responses.create(
        model=model,
        input=prompt + content + "\n" + "Now, please provide a response to the user's question.\n",
    )

    result = response.output_text

    return result
