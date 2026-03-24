"""
PEDSA - Brain-inspired RAG-less Memory Engine

Usage:
    >>> import pedsa
    >>> engine = pedsa.Engine()
    >>> engine.add_feature(1, "rust")
    >>> engine.add_event(100, "Rust 在嵌入式领域取得突破")
    >>> engine.add_edge(1, 100, 0.9)
    >>> engine.compile()
    >>> results = engine.retrieve("Rust 内存安全")
"""

from .pedsa import *  # noqa: F401, F403
