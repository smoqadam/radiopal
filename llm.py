"""Thin LLM wrapper via LiteLLM, so the provider/model is swappable by config.

Pick the model with RADIOPAL_LLM_MODEL in LiteLLM's "<provider>/<model>" form:
  deepseek/deepseek-chat     (default — DeepSeek's latest chat model)
  openai/gpt-4o-mini
  gemini/gemini-1.5-flash
  ollama/llama3              (local, no key)

Credentials follow LiteLLM conventions per provider, e.g. DEEPSEEK_API_KEY,
OPENAI_API_KEY, GEMINI_API_KEY. Nothing else here needs to change to switch.
"""
from __future__ import annotations

import os

import litellm

DEFAULT_MODEL = os.environ.get("RADIOPAL_LLM_MODEL", "deepseek/deepseek-v4-flash")


def generate(
    system: str,
    user: str,
    *,
    model: str | None = None,
    max_tokens: int = 200,
    temperature: float = 0.8,
) -> str:
    """Return the model's reply text for a system+user prompt pair."""
    response = litellm.completion(
        model=model or DEFAULT_MODEL,
        messages=[
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        max_tokens=max_tokens,
        temperature=temperature,
    )
    return response.choices[0].message.content.strip()
