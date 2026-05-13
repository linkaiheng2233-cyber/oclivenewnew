"""轻量 Python OOCP 客户端（标准库 HTTP，无强制第三方依赖）。"""

from .client import OcliveClient, OcliveError

__all__ = ["OcliveClient", "OcliveError"]
