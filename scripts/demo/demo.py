import time
import sys

def p(text, delay=0.8):
    print(text)
    sys.stdout.flush()
    time.sleep(delay)

# Colors
PURPLE = "\033[1;35m"
CYAN = "\033[1;36m"
GREEN = "\033[1;32m"
RESET = "\033[0m"

time.sleep(0.5)
p(f"{PURPLE}[Coordinator]{RESET} Session started. Creating multi-agent workflow tasks...", 1.5)
p(f"{PURPLE}[Coordinator]{RESET} 📝 Task created: 'Research Quantum Computing' (id: 48f2)", 0.5)
p(f"{PURPLE}[Coordinator]{RESET} 📝 Task created: 'Write Article' (id: 9a1b)", 2.0)

p(f"{CYAN}[Alice-Research]{RESET} Found task 'Research Quantum Computing' (id: 48f2). Claiming...", 1.2)
p(f"{CYAN}[Alice-Research]{RESET} 🔒 Acquired exclusive lock on task 48f2.", 2.0)
p(f"{CYAN}[Alice-Research]{RESET} Research complete! Findings: 'Stability improved by 40%'.", 1.0)
p(f"{CYAN}[Alice-Research]{RESET} 📨 Sending payload to inbox of 'Bob-Writer'.", 1.0)
p(f"{CYAN}[Alice-Research]{RESET} ✅ Marked task 48f2 as DONE.", 2.5)

p(f"{GREEN}[Bob-Writer]{RESET} Found task 'Write Article' (id: 9a1b). Claiming...", 1.2)
p(f"{GREEN}[Bob-Writer]{RESET} 🔒 Acquired exclusive lock on task 9a1b. Polling inbox...", 1.8)
p(f"{GREEN}[Bob-Writer]{RESET} 🔔 Inbox new item! Received findings from Alice-Research.", 1.5)
p(f"{GREEN}[Bob-Writer]{RESET} Drafting final article using research context...", 2.5)
p(f"{GREEN}[Bob-Writer]{RESET} ✅ Marked task 9a1b as DONE.", 1.5)

p(f"{PURPLE}[Coordinator]{RESET} Workflow complete! All sessions gracefully checked out.", 2.0)
