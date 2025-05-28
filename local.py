from transformers import AutoTokenizer, AutoModelForCausalLM
from remote import run_remote_response
from shared import prompt, prompt_suffix

model_name = "Qwen/Qwen3-0.6B"

tokenizer = AutoTokenizer.from_pretrained(model_name)
llm = AutoModelForCausalLM.from_pretrained(
    model_name, torch_dtype="auto", device_map="auto"
)


def generate_local_response(content: str, user: str) -> str:
    message = prompt + content + prompt_suffix

    text = tokenizer.apply_chat_template(
        [
            {
                "role": "user",
                "content": message
                + f"Now the user is asking: {user}\n"
                + f"You should be aware that you may be unable to solve this question. If you don't think yourself able to solve the user's problem, please output the special token <remote> instead of returning other sentences.",
            }
        ],
        tokenize=False,
        add_generation_prompt=True,
        enable_thinking=False,  # Switches between thinking and non-thinking modes. Default is True.
    )
    model_inputs = tokenizer([text], return_tensors="pt").to(llm.device)

    # conduct text completion
    generated_ids = llm.generate(**model_inputs, max_new_tokens=1024)
    output_ids = generated_ids[0][len(model_inputs.input_ids[0]) :].tolist()

    result = tokenizer.decode(output_ids, skip_special_tokens=True).strip("\n")

    return result

def generate_response(content: str, user: str) -> str:
    """
    Generate a response based on the content provided.
    This function is a wrapper for generate_local_response.
    """
    result = generate_local_response(content, user)
    if result == "<remote>":
        result = run_remote_response(content, user)
    return result