#!/usr/bin/env python3
"""Ping periódico para mantener despierto el backend desplegado en Render.

Uso básico:
    python scripts/keep_alive.py

Variables opcionales:
    KEEP_ALIVE_URL       URL a consultar. Default: https://agentic-paperwork.onrender.com/health
    KEEP_ALIVE_INTERVAL  Segundos entre requests. Default: 600
    KEEP_ALIVE_TIMEOUT   Timeout por request en segundos. Default: 20
"""

from __future__ import annotations

import os
import time
from datetime import datetime, timezone
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

DEFAULT_URL = "https://agentic-paperwork.onrender.com/health"
DEFAULT_INTERVAL_SECONDS = 600
DEFAULT_TIMEOUT_SECONDS = 20


def env_int(name: str, default: int) -> int:
    raw_value = os.getenv(name)
    if raw_value is None:
        return default

    try:
        value = int(raw_value)
    except ValueError:
        print(f"{name} inválido: {raw_value!r}. Usando {default}.")
        return default

    return max(1, value)


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def ping(url: str, timeout: int) -> None:
    request = Request(
        url,
        headers={
            "User-Agent": "PaperMind-KeepAlive/1.0",
            "Accept": "application/json,text/plain,*/*",
        },
        method="GET",
    )

    started = time.monotonic()
    try:
        with urlopen(request, timeout=timeout) as response:
            elapsed_ms = int((time.monotonic() - started) * 1000)
            body = (
                response.read(200).decode("utf-8", errors="replace").replace("\n", " ")
            )
            print(
                f"[{utc_now()}] OK status={response.status} elapsed_ms={elapsed_ms} body={body}"
            )
    except HTTPError as error:
        elapsed_ms = int((time.monotonic() - started) * 1000)
        print(f"[{utc_now()}] HTTP_ERROR status={error.code} elapsed_ms={elapsed_ms}")
    except URLError as error:
        elapsed_ms = int((time.monotonic() - started) * 1000)
        print(f"[{utc_now()}] URL_ERROR reason={error.reason} elapsed_ms={elapsed_ms}")
    except TimeoutError:
        elapsed_ms = int((time.monotonic() - started) * 1000)
        print(f"[{utc_now()}] TIMEOUT elapsed_ms={elapsed_ms}")


def main() -> None:
    url = os.getenv("KEEP_ALIVE_URL", DEFAULT_URL)
    interval = env_int("KEEP_ALIVE_INTERVAL", DEFAULT_INTERVAL_SECONDS)
    timeout = env_int("KEEP_ALIVE_TIMEOUT", DEFAULT_TIMEOUT_SECONDS)

    print("PaperMind keep-alive iniciado")
    print(f"URL: {url}")
    print(f"Intervalo: {interval} segundos")
    print(f"Timeout: {timeout} segundos")

    while True:
        ping(url, timeout)
        time.sleep(interval)


if __name__ == "__main__":
    main()
