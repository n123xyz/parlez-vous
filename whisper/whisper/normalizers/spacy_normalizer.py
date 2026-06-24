import spacy

# Load the spaCy model once when the module is imported.
# This assumes the model is installed (we'll do that in the Dockerfile).
try:
    nlp = spacy.load("en_core_web_sm")
except IOError:
    # This is a fallback for local testing.
    # The Docker build should handle this.
    print("Downloading spaCy model 'en_core_web_sm'...")
    spacy.cli.download("en_core_web_sm")
    nlp = spacy.load("en_core_web_sm")


class SpacyNormalizer:
    def __call__(self, s: str) -> str:
        """
        Applies spaCy normalization:
        1. Lowercases
        2. Lemmatizes (reduces words to their root form)
        3. Removes stop words (like 'a', 'the', 'is')
        4. Removes punctuation
        """
        # Process the string with spaCy's pipeline
        doc = nlp(s.lower())
        
        # Create a list of tokens, applying all filters
        tokens = [
            token.lemma_
            for token in doc
            if (
                not token.is_stop and    # remove stop words
                not token.is_punct and   # remove punctuation
                not token.is_space       # remove whitespace tokens
            )
        ]
        
        # Join the cleaned tokens back into a single string
        return " ".join(tokens)
