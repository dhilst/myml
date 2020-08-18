from typing import *

# shift reduce parser

rules = {
    "S": "S + S".split(),
    "S": "S * S".split(),
    "S": "id".split(),
}

from pprint import pprint


def parse(input: List[str]) -> bool:
    # (1) S : S + S
    # (2) S : S * S
    # (3) S : id
    stack = []  # type: ignore

    while True:
        if len(stack) > 0:
            if stack[-1] == "id":
                # reduce by (3)
                t = stack.pop()
                stack.append("S")
                print("reduce by 3")
                pprint(stack)
            elif len(stack) >= 3:
                t1, t2, t3 = stack.pop(), stack.pop(), stack.pop()
                if t1 == "S" and t2 == "*" and t3 == "S":
                    stack.append("S")
                    print("reduce by 2")
                    pprint(stack)
                elif t1 == "S" and t2 == "+" and t3 == "S":
                    stack.append("S")
                    print("reduce by 1")
                    pprint(stack)
                else:
                    raise RuntimeError(f"{t1} {t2} {t3}")
            else:
                t = input.pop(0)
                stack.append(t)
                print(f"shift {t}")
                pprint(stack)
        else:
            t = input.pop(0)
            stack.append(t)
            print(f"shift {t}")
            pprint(stack)

        if len(input) == 0:
            if len(stack) == 1:
                return stack


assert parse("id + id * id".split())
