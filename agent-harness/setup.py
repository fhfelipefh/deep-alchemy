from setuptools import setup, find_namespace_packages

setup(
    name="cli-anything-deep-alchemy",
    version="0.1.0",
    description="Agent-native CLI harness for Deep Alchemy (Bevy Rust Game Engine)",
    long_description=open("cli_anything/deep_alchemy/README.md", encoding="utf-8").read()
    if open("cli_anything/deep_alchemy/README.md", encoding="utf-8")
    else "",
    long_description_content_type="text/markdown",
    author="Deep Alchemy Contributors",
    packages=find_namespace_packages(include=["cli_anything.*"]),
    install_requires=[
        "click>=8.0",
        "prompt-toolkit>=3.0",
    ],
    extras_require={
        "visual": ["Pillow>=9.0", "numpy>=1.20"],
    },
    entry_points={
        "console_scripts": [
            "cli-anything-deep-alchemy=cli_anything.deep_alchemy.deep_alchemy_cli:main",
            "deep-alchemy-cli=cli_anything.deep_alchemy.deep_alchemy_cli:main",
        ],
    },
    python_requires=">=3.10",
)
