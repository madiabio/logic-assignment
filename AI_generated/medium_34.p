fof(fact, axiom, p0(a)).
fof(step1, axiom, ! [X] : (p1(X) => p2(X))).
fof(step2, axiom, ! [X] : (p2(X) => p3(X))).
fof(step3, axiom, ! [X] : (p3(X) => p4(X))).
fof(step4, axiom, ! [X] : (p4(X) => p5(X))).
fof(dist0, axiom, ! [X] : (q0(X) => q1(X))).
fof(dist1, axiom, ! [X] : (q1(X) => q2(X))).
fof(goal, conjecture, p5(a)).
