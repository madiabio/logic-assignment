from __future__ import annotations

import re
from dataclasses import dataclass


class UnsupportedFormula(ValueError):
    """Raised when a FOLIO formula is outside the bridge's supported subset."""


@dataclass(frozen=True)
class Token:
    kind: str
    value: str


class Lexer:
    def __init__(self, source: str):
        self.source = source
        self.pos = 0

    def tokenize(self) -> list[Token]:
        tokens: list[Token] = []
        while self.pos < len(self.source):
            ch = self.source[self.pos]
            if ch.isspace() or ch == ",":
                self.pos += 1
                continue
            if self.source.startswith("<=>", self.pos):
                tokens.append(Token("IFF", "<=>"))
                self.pos += 3
                continue
            if self.source.startswith("=>", self.pos) or self.source.startswith("->", self.pos):
                tokens.append(Token("IMPLIES", self.source[self.pos : self.pos + 2]))
                self.pos += 2
                continue
            if ch in "()":
                tokens.append(Token(ch, ch))
                self.pos += 1
                continue
            if ch in {"∀", "!"}:
                tokens.append(Token("FORALL", ch))
                self.pos += 1
                continue
            if ch in {"∃", "?"}:
                tokens.append(Token("EXISTS", ch))
                self.pos += 1
                continue
            if ch in {"¬", "~"}:
                tokens.append(Token("NOT", ch))
                self.pos += 1
                continue
            if ch in {"∧", "&"}:
                tokens.append(Token("AND", ch))
                self.pos += 1
                continue
            if ch in {"∨", "|"}:
                tokens.append(Token("OR", ch))
                self.pos += 1
                continue
            if ch in {"→", "⟶"}:
                tokens.append(Token("IMPLIES", ch))
                self.pos += 1
                continue
            if ch in {"↔", "⟷"}:
                tokens.append(Token("IFF", ch))
                self.pos += 1
                continue
            if ch in {"⊕", "⊻"}:
                tokens.append(Token("XOR", ch))
                self.pos += 1
                continue
            if ch == "=":
                raise UnsupportedFormula("equality is not supported")
            if ch.isalnum() or ch == "_":
                start = self.pos
                while self.pos < len(self.source) and (
                    self.source[self.pos].isalnum() or self.source[self.pos] in {"_", "-"}
                ):
                    self.pos += 1
                word = self.source[start : self.pos]
                lowered = word.lower()
                aliases = {
                    "forall": "FORALL",
                    "exists": "EXISTS",
                    "not": "NOT",
                    "and": "AND",
                    "or": "OR",
                    "xor": "XOR",
                }
                tokens.append(Token(aliases.get(lowered, "IDENT"), word))
                continue
            raise UnsupportedFormula(f"unsupported character {ch!r}")
        tokens.append(Token("EOF", ""))
        return tokens


class FolParser:
    def __init__(self, source: str):
        self.tokens = Lexer(source).tokenize()
        self.index = 0
        self.bound_vars: list[str] = []

    def parse(self) -> str:
        formula = self.parse_iff()
        if self.peek().kind != "EOF":
            raise UnsupportedFormula(f"unexpected token {self.peek().value!r}")
        return formula

    def peek(self) -> Token:
        return self.tokens[self.index]

    def pop(self, kind: str | None = None) -> Token:
        token = self.peek()
        if kind is not None and token.kind != kind:
            raise UnsupportedFormula(f"expected {kind}, got {token.value!r}")
        self.index += 1
        return token

    def parse_iff(self) -> str:
        left = self.parse_implies()
        while self.peek().kind == "IFF":
            self.pop("IFF")
            right = self.parse_implies()
            left = f"({left} <=> {right})"
        return left

    def parse_implies(self) -> str:
        left = self.parse_xor()
        if self.peek().kind == "IMPLIES":
            self.pop("IMPLIES")
            right = self.parse_implies()
            return f"({left} => {right})"
        return left

    def parse_xor(self) -> str:
        left = self.parse_or()
        while self.peek().kind == "XOR":
            self.pop("XOR")
            right = self.parse_or()
            left = f"(({left} | {right}) & ~(({left} & {right})))"
        return left

    def parse_or(self) -> str:
        left = self.parse_and()
        while self.peek().kind == "OR":
            self.pop("OR")
            right = self.parse_and()
            left = f"({left} | {right})"
        return left

    def parse_and(self) -> str:
        left = self.parse_unary()
        while self.peek().kind == "AND":
            self.pop("AND")
            right = self.parse_unary()
            left = f"({left} & {right})"
        return left

    def parse_unary(self) -> str:
        token = self.peek()
        if token.kind == "NOT":
            self.pop("NOT")
            return f"~({self.parse_unary()})"
        if token.kind in {"FORALL", "EXISTS"}:
            quantifier = "!" if token.kind == "FORALL" else "?"
            self.pop(token.kind)
            variables = self.parse_quantifier_variables()
            previous_len = len(self.bound_vars)
            self.bound_vars.extend(variables)
            body = self.parse_unary()
            del self.bound_vars[previous_len:]
            joined = ", ".join(tptp_variable(v) for v in variables)
            return f"({quantifier} [{joined}] : {body})"
        if token.kind == "(":
            self.pop("(")
            inner = self.parse_iff()
            self.pop(")")
            return inner
        return self.parse_atom()

    def parse_quantifier_variables(self) -> list[str]:
        variables = [self.pop("IDENT").value]
        while self.peek().kind == "IDENT":
            variables.append(self.pop("IDENT").value)
        return variables

    def parse_atom(self) -> str:
        name = sanitize_lower(self.pop("IDENT").value)
        if self.peek().kind != "(":
            return name
        self.pop("(")
        args: list[str] = []
        if self.peek().kind != ")":
            args.append(self.parse_term())
            while self.peek().kind not in {")", "EOF"}:
                args.append(self.parse_term())
        self.pop(")")
        return f"{name}({', '.join(args)})"

    def parse_term(self) -> str:
        name = self.pop("IDENT").value
        if self.peek().kind == "(":
            functor = sanitize_lower(name)
            self.pop("(")
            args: list[str] = []
            if self.peek().kind != ")":
                args.append(self.parse_term())
                while self.peek().kind not in {")", "EOF"}:
                    args.append(self.parse_term())
            self.pop(")")
            return f"{functor}({', '.join(args)})"
        if name in self.bound_vars:
            return tptp_variable(name)
        return sanitize_lower(name)


def sanitize_lower(name: str) -> str:
    clean = re.sub(r"[^A-Za-z0-9_]", "_", name).strip("_").lower()
    if not clean:
        raise UnsupportedFormula(f"empty symbol after sanitizing {name!r}")
    if clean[0].isdigit():
        clean = f"c_{clean}"
    return clean


def tptp_variable(name: str) -> str:
    clean = re.sub(r"[^A-Za-z0-9_]", "_", name).strip("_")
    if not clean:
        raise UnsupportedFormula(f"empty variable after sanitizing {name!r}")
    clean = clean[0].upper() + clean[1:]
    if not clean[0].isupper():
        clean = f"V_{clean}"
    return clean


def convert_fol_formula(formula: str) -> str:
    if not isinstance(formula, str) or not formula.strip():
        raise UnsupportedFormula("empty formula")
    return FolParser(formula).parse()
