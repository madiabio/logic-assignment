fof(fact, axiom, p0(a)).
fof(step0, axiom, ! [X] : (p0(X) => p1(X))).
fof(step1, axiom, ! [X] : (p1(X) => p2(X))).
fof(dist0, axiom, ! [X] : (q0(X) => q1(X))).
fof(goal, conjecture, p2(a)).
