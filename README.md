Ce fichier contient les *spécifications* de la prod. Il contient en l’occurence tout le storyboard,
les détails de direction, le choix du thème, les couleurs, la synchronisation, etc.

# Storyboard

Contient le storyboard de la prod, c’est à dire le thème et les grandes lignes.

La prod devra avoir un format de mini demo. **Au maximum deux minutes de runtime**. En deux minutes,
il faut avoir des scènes rapides et fortement remplies.

## Brainstorming

En ce moment j’aime bien bosser avec les splines et le blur / gloom. J’ai envie de faire une énorme
scène avec plein de splines qui « éclairent » et bougent au rythme de la musique. Je verrais bien
une scène avec des particules en suspension, genre poussière, dans la même scène que les cubes
connectés par des lasers. Ça nous fait déjà deux scènes. J’ai aussi la scène avec le plan tessellé
dont les edges seraient gloomy. Ça fait trois scènes. On pourrait alterner entre ces scènes
rapidement, et il nous faudrait une scène finale qu’on afficherait avec du son style une brise et
l’écran qui « sautille » et « grésille ». Genre une scène au dessus des nuages avec un logo ou quoi.

Donc on part du principe qu’on a ces scènes :

- spline moving mountains
- lasers cube and dirt particles
- tessellated plane
- finale skies

Il est important de comprendre qu’il n’y a pas d’ordre dans la « *lecture* » de ces scènes, et qu’on
va les entrelacer les unes les autres pour obtenir une démo dynamique. Le must serait d’avoir un
*truc* par scène. Par exemple, une ambiance bien particulière pour chaque scène, voire une histoire,
et d’arriver à les connecter malgré tout. Ça me semble compliqué pour une première demo.

## Story

J’ai envie de m’exprimer au sujet de ma personnalité. J’ai envie d’expliquer qu’aujourd’hui, on
passe beaucoup de temps à chercher le bonheur et les sentiments positifs et à absolument éviter les
sentiments négatifs. Il **faut** éviter les sentiments négatifs. Pourquoi ? Les gens ont des
histoires d’amour, se séparent et souffrent. Puis ils n’osent plus se relancer dans une nouvelle
relation de peur d’être blessés comme ils l’ont été par le passé. C’est la mauvaise réaction à la
vie.

La tristesse et la peur sont des sentiments forts. Il ne faut pas chercher à les écarter de sa vie.
Il faut savoir composer avec les sentiments positifs et les sentiments négatifs. C’est primordial.

## Spline moving mountains

## Lasers cube & dirt particles

## Tessellation plane

## Finale

# Sync

Cette partie contient les clefs de synchronisation avec la musique.

# Workflow 

Cette partie permet de définir le workflow; d’éventuels plannings; les tâches.

---

Il faut pouvoir coder rapidement et prototyper. On doit savoir comment organiser le code de la démo.

Est-ce que l’on fait *“un module Rust par scène”* ? Je pense que c’est trop limité de faire ça.
Qu’est-ce qu’une scène ? Il nous faut du code qui va *driver* la démo.

Driver la démo, ça veut simplement dire quoi faire quand. Par exemple, dire « à ce moment là, on
lance ce bout de code ». On a deux types de synchronisation du coup :

- la synchronisation au niveau du code
- la synchronisation au niveau des assets

## Code sync

La synchronisation au niveau du code fait référence au fait qu’il faut coder les scènes et dire
comment le code qui les compose s’exécute. Sur le long terme, ce genre de synchro n’existera plus
au profit de synchronisation data-driven.

La notion de scène étant floue, il faut définir un moyen de passer d’une scène à l’autre au niveau
du code. La changement de scène étant un évènement ponctuel, discret – et non pas continu, on
gaspillerait du temps à sampler une `AnimParam` pour ça. Il faut donc un autre système, beaucoup
plus simple. On peut partir d’un modèle à base de `Pulse`, par exemple. Dire qu’un `Pulse`
représente un évènement qui a lieu à un moment donné. Ainsi, soit cet évènement n’a pas eu lieu, soit
il a déjà eu lieu. Dès lors qu’un `Pulse` dépasse sa valeur de déclenchement, sa closure est appelée
une seule fois et n’est plus jamais appelée. C’est le principe de cet objet.

Seulement, ce truc là nous permet de muter uniquement un état extern au pulse. Ce n’est pas
forcément très safe ni très agréable à utiliser. En réalité ce que l’on voudrait, c’est une fonction
qui bouffe le temps et qui change de temps en temps. C’est des `Behavior` + `switch` – du FRP.

## Data-driven sync

Ce type de synchro est simple à comprendre. Certains assets adoptent un comportement continu dans
le temps. C’est le cas typique d’une caméra, dont sa position, orientation et projection va changer
au cours du temps, ou le cas d’un objet qui bouge. Pour synchroniser ces objets, on va utiliser
des `AnimParam`. Ces objets doivent être accessibles en lecture seule dans toutes les scènes.

## Structure de données de la demo

Un problème que j’ai régulièrement est de savoir quelle structure de données utiliser pour
représenter les différents objets de la prod. Étant donné que ce code est très très muable – c’est à
dire qu’on passe notre temps à ajouter des champs, des objets, à changer des trucs, etc. – avoir
une structure de données nommée n’est pas très agréable.

En Haskell, j’utilisais un principe très con : les scopes. Les scopes permettent de partager des
objets sans avoir à utiliser des structures. Au lieu de :

```
struct Demo {
	camera: …,
	mesh0: …,
	mesh1: …,
	…
}
```

Qui ne scale pas du tout car on doit modifier cette structure à chaque fois que l’on ajoute / enlève
des trucs, on fait plutôt ça :

```
fn init(…) -> … {
	let camera = …;
	let mesh0 = …;
	let mesh1 = …;

	Box::new(move |time| {
		// ici on déplace l’environnement, donc on a en quelques sortes créer notre structure anonyme
		// que l’on passe à la closure ; c’est bien meilleur de laisser le compilateur faire ça pour
		// nous
	})
}
```

L’avantage de cette technique est que ça scale bien mieux. Pas besoin de modifier une structure
pour rajouter des objets : on peut juste les rajouter et jouer avec.

Cette manière de faire met aussi en évidence un pattern que j’utilisais en Haskell dans quaazar : on
a une fonction d’initialisation qui va retourner une closure définissant comment la démo va se
comporter. C’est très pratique car ça nous permet de partager les ressources efficacement tout en
ayant un boilerplate minimal.

