import sys
from collections import namedtuple
from re import compile
import inspect
from typing import *

# shift reduce parser

from pprint import pprint


def shift(stack, input):
    t = input.pop(0)
    stack.append(t)
    print(f"shift {t}")


def cmpt(a, b):
    try:
        return a[0] == b
    except IndexError:
        return False


Token = namedtuple("Token", "type value")
TokenRule = namedtuple("Token", "type matcher mapper")
GrammarRule = namedtuple("GrammarRule", "lhs rhss")

TOKENS = [
    TokenRule("PLUS", compile("\+"), str),
    TokenRule("MUL", compile("\*"), str),
    TokenRule("INT", compile("\d+"), int),
    TokenRule("SYM", compile(r"[a-z]\w*"), str),
]


GRAMMAR = [
    """expr   : expr PLUS term
              | term""",
    """term   : term MUL atom
              | atom""",
    """atom   : INT
              | SYM""",
]


def lex(input, tokens):
    output = []
    while input:
        input = input.lstrip()
        for t in tokens:
            m = t[1].match(input)
            if m is not None:
                input = input[m.end() :]
                output.append(Token(t[0], m.group()))
    return output


def parse_gramar(grammar):
    output = []
    for rule in grammar:
        a, b = map(str.strip, rule.split(":"))
        b = [b.strip() for b in b.split("|")]
        output.append(GrammarRule(a, b))
    return output


class ParseError(Exception):
    pass


class RuleMatched(Exception):
    pass


Handler = Any
Stack = List[Token]


def print_token(token, indent=1):
    if isinstance(token, Token):
        print(f"  " * indent + f" {token.type}")
        for v in token.value:
            print_token(v, indent + 2)
    else:
        print(f"  " * indent + f" {token}")


def reduce(lhs: str, rhs: str, stack: Stack):
    arity = len(rhs)
    parms = stack[-arity:]
    del stack[-arity:]
    print(f"reduce {lhs} <- {rhs}")
    t = Token(lhs, parms)
    print_token(t)
    stack.append(t)


def match(subrule: str, stack: Stack) -> bool:
    """
    >>> match("PLUS", [Token("PLUS","+")])
    True

    >>> match("term PLUS term", [Token("term", ""), Token("PLUS","+"), Token("term", "")])
    True
    """
    terms = subrule.split()
    arity = len(terms)
    if arity > len(stack):
        return False
    stackterms = stack[-arity:]
    for t1, t2 in zip(terms, stackterms):
        if t1 != t2.type:
            return False
    return True


def parse2(grammar: List[str], input: List[Token]) -> Token:
    stack: Stack = []

    shift(stack, input)
    grammar_ = parse_gramar(grammar)

    while True:
        try:
            for lhs, rhs in grammar_:
                for subrule in rhs:
                    if match(subrule, stack):
                        reduce(lhs, subrule, stack)
                        raise RuleMatched
            if not input and stack:
                raise ParseError(stack)
            shift(stack, input)
        except RuleMatched:
            pass

        if not input and len(stack) == 1:
            return stack[0]


tokens = lex("a + b * 2", TOKENS)
result = parse2(GRAMMAR, tokens)
print("Result")
print_token(result)
# sys.exit(1)


def parse(input: List[str]) -> List[Any]:
    # (1) S : S + S
    # (2) S : S * S
    # (3) S : id
    stack: List[Any] = []

    # shift first token
    # try rules or shift

    shift(stack, input)

    while True:
        if stack[-1] == "id":
            h = stack.pop()
            stack.append(("S", h))
            print("reduce S <- id")
        elif len(stack) >= 3:
            a, b, c = stack[-1], stack[-2], stack[-3]
            if cmpt(a, "S") and b == "+" and cmpt(c, "S"):
                print("reduce S <- S + S")
                stack.pop()
                stack.pop()
                stack.pop()
                stack.append(("S", b, a, c))
            elif cmpt(a, "S") and b == "*" and cmpt(c, "S"):
                print("reduce S <- S * S")
                stack.pop()
                stack.pop()
                stack.pop()
                stack.append(("S", b, a, c))
            else:
                print(stack, a[0], b, c[0])
        else:
            shift(stack, input)

            if len(input) == 0:
                if len(stack) == 1:
                    return stack


##meprint("STACK", parse("id + id * id".split()))
