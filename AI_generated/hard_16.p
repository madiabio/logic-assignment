fof(isa0, axiom, ! [X] : (entity(X) => organism(X))).
fof(isa1, axiom, ! [X] : (organism(X) => animal(X))).
fof(isa2, axiom, ! [X] : (animal(X) => mammal(X))).
fof(isa3, axiom, ! [X] : (mammal(X) => primate(X))).
fof(isa4, axiom, ! [X] : (primate(X) => human(X))).
fof(isa5, axiom, ! [X] : (human(X) => person(X))).
fof(isa6, axiom, ! [X] : (person(X) => scholar(X))).
fof(isa7, axiom, ! [X] : (scholar(X) => student(X))).
fof(isa8, axiom, ! [X] : (student(X) => teacher(X))).
fof(inst, axiom, entity(socrates)).
fof(goal, conjecture, teacher(socrates)).
