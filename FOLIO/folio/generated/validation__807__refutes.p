fof(premise_1,axiom,(! [X] : (bakedsweet(X) => ~(spicy(X))))).
fof(premise_2,axiom,(! [X] : (cupcake(X) => bakedsweet(X)))).
fof(premise_3,axiom,(! [X] : (malahotpot(X) => spicy(X)))).
fof(premise_4,axiom,(! [X] : ((product(X) & from(X, bakedbymelissa)) => cupcake(X)))).
fof(premise_5,axiom,((spicy(driedthaichili) | malahotpot(driedthaichili)) | ~(bakedsweet(driedthaichili)))).
fof(conclusion_negated,conjecture,~(malahotpot(driedthaichili))).
