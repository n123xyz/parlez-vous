from .basic import BasicTextNormalizer as BasicTextNormalizer
from .english import EnglishTextNormalizer as EnglishTextNormalizer
from .spacy_normalizer import SpacyNormalizer

def get_normalizer(name: str):
    """
    Returns an instance of the specified normalizer.
    """
    if name == "basic":
        return BasicTextNormalizer()
    if name == "english":
        return EnglishTextNormalizer()
    if name == "spacy":
        return SpacyNormalizer()
    
    # Fallback for None or unknown
    return None

__all__ = [
    "BasicTextNormalizer",
    "EnglishTextNormalizer",
    "SpacyNormalizer",
    "get_normalizer",
]
