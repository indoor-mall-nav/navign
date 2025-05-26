from transformers import AutoTokenizer, AutoModelForCausalLM

prompt = '<|im_start|>/no_think You are talking to a person who is unable to get the whole sight to the room. You will be given a list of objects and their 3D coordinates. Your task is to describe the scene with your text to a blind in the room. Suppose the person is in (0,0,0) in this scene. You should not involve any coordinate, including exact number, but use "far" or "near."'  # basic system prompt
prompt_suffix = "\n<|im_end|><|im_start|>\n"

model_name = "Qwen/Qwen3-0.6B"

tokenizer = AutoTokenizer.from_pretrained(model_name)
llm = AutoModelForCausalLM.from_pretrained(
    model_name, torch_dtype="auto", device_map="auto"
)


def generate_local_response(content: str) -> str:
    message = prompt + content + prompt_suffix

    text = tokenizer.apply_chat_template(
        [
            {
                "role": "user",
                "content": message
                + f"Now the user is asking: {input()}\n"
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
