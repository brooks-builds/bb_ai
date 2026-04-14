FROM rust:latest

ENV LLM_BASE_URL=https://openrouter.ai/api/v1
ENV LLM_API_KEY=2q3984y7jsrlethdaireshaiernsht
ENV LLM_MODEL=anthropic/claude-haiku-4.5

VOLUME ["/code"]
WORKDIR /code

CMD ["bash"]
