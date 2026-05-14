fof(fact, axiom, p0(a)).
fof(step0, axiom, ! [X] : (p0(X) => p1(X))).
fof(step1, axiom, ! [X] : (p1(X) => p2(X))).
fof(step3, axiom, ! [X] : (p3(X) => p4(X))).
fof(dist0, axiom, ! [X] : (q0(X) => q1(X))).
fof(dist1, axiom, ! [X] : (q1(X) => q2(X))).
fof(dist2, axiom, ! [X] : (q2(X) => q3(X))).
fof(goal, conjecture, p4(a)).
