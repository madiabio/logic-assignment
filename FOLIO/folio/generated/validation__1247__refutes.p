fof(premise_1,axiom,(! [X] : ((student(X) & workin(X, library)) => ~(from(X, computersciencedepartment))))).
fof(premise_2,axiom,(! [X] : (((student(X) & have(X, parttimejob)) & offeredby(y, university)) => workin(X, library)))).
fof(premise_3,axiom,(! [X] : ((student(X) & take(X, databasecourse)) => from(X, computersciencedepartment)))).
fof(premise_4,axiom,(! [X] : ((student(X) & instructedby(X, professordavid)) => take(X, databasecourse)))).
fof(premise_5,axiom,(! [X] : ((student(X) & workin(X, lab)) => instructedby(X, professordavid)))).
fof(premise_6,axiom,(student(james) & workin(james, lab))).
fof(premise_7,axiom,(~((? [X] : ((parttimejob(X) & have(james, X)) & offeredby(X, computersciencedepartment)))) & ~(workin(james, lab)))).
fof(conclusion_negated,conjecture,~(take(james, databasecourse))).
