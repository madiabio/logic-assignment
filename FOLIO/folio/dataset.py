from __future__ import annotations

DATASET_NAME = "yale-nlp/FOLIO"


def load_folio_dataset(split: str):
    try:
        from datasets import load_dataset
    except ImportError as err:
        raise SystemExit(
            "Install the Hugging Face datasets package to download FOLIO: pip install datasets"
        ) from err

    try:
        dataset = load_dataset(DATASET_NAME)
    except Exception as err:
        raise SystemExit(
            "Could not load gated Hugging Face dataset yale-nlp/FOLIO. "
            "Request access on Hugging Face and run `huggingface-cli login`, then retry."
        ) from err

    if split == "all":
        return [(name, dataset[name]) for name in dataset.keys()]
    if split not in dataset:
        raise SystemExit(
            f"Dataset split {split!r} is not available. Available: {', '.join(dataset.keys())}"
        )
    return [(split, dataset[split])]
