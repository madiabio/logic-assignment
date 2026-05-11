fof(premise_1,axiom,(! [X] : ((employee(X) & schedule(X, meeting, customers)) => appearin(X, company)))).
fof(premise_2,axiom,(! [X] : ((employee(X) & haslunch(X, company)) => schedule(X, meeting, customers)))).
fof(premise_3,axiom,(! [X] : (employee(X) => ((haslunch(X, company) | haslunch(X, home)) & ~((haslunch(X, company) & haslunch(X, home))))))).
fof(premise_4,axiom,(! [X] : ((employee(X) & haslunch(X, home)) => work(X, home)))).
fof(premise_5,axiom,(! [X] : ((employee(X) & ~(in(X, homecountry))) => work(X, home)))).
fof(premise_6,axiom,(! [X] : (manager(X) => ~(work(X, home))))).
fof(premise_7,axiom,~(((manager(james) | appearin(james, company)) & ~((manager(james) & appearin(james, company)))))).
fof(conclusion_negated,conjecture,~(~(haslunch(james, company)))).
