from typing_extensions import TypedDict
import requests
from bs4 import BeautifulSoup


class Content(TypedDict):
    title: str
    url: str


headers = {
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",  # noqa
    "Accept-Language": "en-TH,en;q=0.9,th-TH;q=0.8,th;q=0.7,en-GB;q=0.6,en-US;q=0.5",  # noqa
    "Cache-Control": "max-age=0",
    "Connection": "keep-alive",
    "Sec-Fetch-Dest": "document",
    "Sec-Fetch-Mode": "navigate",
    "Sec-Fetch-Site": "none",
    "Sec-Fetch-User": "?1",
    "Upgrade-Insecure-Requests": "1",
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",  # noqa
    "sec-ch-ua": '"Google Chrome";v="125", "Chromium";v="125", "Not.A/Brand";v="24"',  # noqa
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": '"macOS"',
}

elem: BeautifulSoup


def extract_pop_up_table(pop_up_id: str) -> list[Content]:
    """Extract PDF files from pop up.

    Args:
        pop_up_id (str): Pop up ID

    Returns:
        list[Content]: Output PDF URLs
    """
    pop_up = elem.find("div", {"id": pop_up_id})
    table_of_content: list[Content] = []
    for header in pop_up.find_all("h4"):
        title = header.get_text().strip()

        for sub_elem in header.next_elements:
            if sub_elem.name == "a":
                url = sub_elem.get("href")
                table_of_content.append({"title": title, "url": url})
                break
    return table_of_content


def extract_table() -> list[Content]:
    """Extract PDF files from the SFC website.

    Returns:
        list[Content]: Output PDF URLs
    """
    global elem
    web_response = requests.get(
        "https://www.sfc.hk/en/Rules-and-standards/Codes-and-guidelines/Codes",
        headers=headers,
    )
    web_response.raise_for_status()

    elem = BeautifulSoup(web_response.text, "lxml")

    table = elem.find("table")

    code_table = [tr.find_all("td")[:-1] for tr in table.find_all("tr")[1:]]
    table_of_content: list[Content] = []

    for row in code_table:
        title, href = row
        title = title.get_text().strip()
        url = href.a.get("href").strip()
        if url != "#":
            content = {"title": title, "url": url}
            table_of_content.append(content)
        else:  # Found pop up
            pop_up_id = href.a.get("data-popup-id").lstrip("#")
            contents = extract_pop_up_table(pop_up_id)
            table_of_content += contents
    return table_of_content


if __name__ == "__main__":
    table_of_contents = extract_table()
    print(table_of_contents)
