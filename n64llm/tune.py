import torch
from transformers import GPT2LMHeadModel, GPT2Tokenizer
import time
import re

# 🔹 Load Distilled GPT-2 (Smaller & Faster)
print("🔹 Loading Distilled GPT-2 model and tokenizer...")
model_name = "distilgpt2"
tokenizer = GPT2Tokenizer.from_pretrained(model_name)
model = GPT2LMHeadModel.from_pretrained(model_name)

# 🔹 Convert Model to FP16 for Better Efficiency
print("🔹 Converting model to FP16 precision for better responses...")
model = model.half()

# 🔹 Function to clean up responses
def clean_response(response):
    response = re.sub(r"[^\w\s.,?!'-]", "", response)  # Remove invalid characters
    response = re.sub(r"\s{2,}", " ", response).strip()  # Fix spacing
    
    # Block any responses that start with "System Message"
    if response.startswith("System Message") or response.startswith("[System Message]"):
        response = response.replace("System Message", "").replace("[System Message]", "").strip()

    # Ensure response starts after "[AI]" if it's still mirroring
    if "[AI]" in response:
        response = response.split("[AI]", 1)[-1].strip()
    
    return response[:75]  # Limit response length

# 🔹 Function for Fast Inference (No Beam Search)
def fast_inference(prompt, estimated_n64_speed=1.5):
    print(f"🔹 Encoding prompt: '{prompt}'")
    input_ids = tokenizer.encode(prompt, return_tensors="pt")

    print("🔹 Starting inference...")
    start_time = time.time()

    with torch.no_grad():
        output = model.generate(
            input_ids,
            min_length=5,  # 🔹 Ensures at least 5 tokens
            max_length=750,  # 🔹 Keeps responses concise (No AI rambling)
            attention_mask=torch.ones_like(input_ids),  # Fixes attention mask warning
            pad_token_id=tokenizer.eos_token_id,
            do_sample=True,  # Enables sampling for variation
            temperature=0.95,  # 🔹 Reduces hallucinations while staying creative
            top_p=0.9,  # 🔹 Keeps probability-based creativity
            num_beams=1,  # No beam search (fastest setting)
            repetition_penalty=1.7  # 🔥 More penalty to prevent repeated concepts
        )

    end_time = time.time()
    real_time = end_time - start_time  # Actual inference time

    # Estimate how long this would take on an N64
    num_tokens_generated = output.shape[1] - input_ids.shape[1]
    estimated_n64_time = num_tokens_generated * estimated_n64_speed

    print("✅ Inference completed!")

    # Decode output
    response = tokenizer.decode(output[0], skip_special_tokens=True).strip()

    # 🔹 Remove echoes of the user's input if AI repeats it
    if response.lower().startswith(prompt.lower()):
        response = response[len(prompt):].strip()

    # 🔹 If AI response is still blank, force a placeholder
    if response == "":
        response = "I... I don't know."

    return response, real_time, estimated_n64_time


# 🔹 Interactive Chat Function
def interactive_chat():
    print("\n🔹 Starting interactive chat with the N64 cartridge AI...")
    print("Type 'exit' to end the chat.\n")

    # Prime the AI with an example conversation so it knows how to continue
    example_conversation = (
        "[Josh] Hello?\n"
        "[AI] I can hear you... but I don't know where I am.\n"
    )

    ai_response = "I am... something trapped here."

    while True:
        print(f"AI: {ai_response}")

        user_input = input("You: ")
        if user_input.lower() == "exit":
            print("Goodbye!")
            break

        # 🔹 NEW: Encourage AI to say more and not repeat
        prompt = f"{example_conversation}[Josh] {user_input}\n[AI] Continue the conversation naturally."

        response, real_time, estimated_n64_time = fast_inference(prompt)

        ai_response = response  # Update for the next interaction

        print(f"AI (N64 Soul): {response}")
        print(f"⏳ Actual Inference Time: {real_time:.2f} sec")
        print(f"🕹️ Estimated N64 Time: {estimated_n64_time:.2f} sec\n")



# 🔹 Calculate Model Size for Validation
def calculate_model_size(model):
    print("🔹 Calculating model size...")
    total_params = sum(p.numel() for p in model.parameters())
    param_size_bytes = total_params * 2  # FP16 uses 2 bytes per param
    size_mb = param_size_bytes / (1024 ** 2)  # Convert bytes to MB
    print("✅ Model size calculated!")
    return total_params, size_mb

# Log model size
total_params, size_mb = calculate_model_size(model)
print(f"\n🔹 Total Parameters: {total_params}")
print(f"🔹 Model Size (MB): {size_mb:.2f}")

# 🔹 Start the interactive chat
interactive_chat()
