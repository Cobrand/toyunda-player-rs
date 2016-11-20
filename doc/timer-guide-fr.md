# Guide du petit timer, édition française

Chaque "time" possède 3 fichiers clés :

* Un fichier vidéo, sous la forme d'un fichier de préférence webm, mais peut aussi être mp4 ou avi.
* Un fichier de sous titre, stocké dans un fichier json
* Un fichier de métadata, stocké dans un fichier yaml

## Fichier vidéo

Ce fichier s'explique lui même, c'est un fichier contenant la vidéo de la musique timée.
Le format webm est préféré, car il est limité en nombre de formats vidéo et audio possible,
et ces formats en particulier sont plus performants en moyenne que les autres formats possibe dans un conteneur mp4 par exemple.

Pour le format audio, choisir pour l'instant le format vorbis au lieu du format opus.
Cela viendra peut être à changer, mais un format uniforme est préféré pour le lecteur, ce serait dommage qu'une partie des vidéos
soient sans son car l'un des deux formats audio utilisé n'est pas supporté.

Pour l'instant opus est plutôt bien supporté, mais le format reste assez jeune et les gains de performance comparé à vorbis ne sont pas assez conséquents pour en valoir le risque.

Pour le format vidéo, vp9 est le format souhaité. vp8 est acceptable, mais vp9 offre de meilleures performances,
alors pourquoi s'en passer ?

### Convertir une vidéo sous ffmpeg

`ffmpeg -i video_entree.mp4 -c:v libvpx-vp9 -crf 25 -b:v 650KB video_sortie.webm`

Les deux valeurs à retenir sont `25` et `650KB`. 25 Correspond à la qualité globale de la vidéo : plus ce nombre est petit, plus la vidéo est proche de l'originale.
Cette valeur peut aller jusqu'à 64, mais la vidéo est très dégradée à ce niveau. En général, une valeur de 25-30 est suffisant pour maintenir une bonne qualité visuelle dans les animes.

Si la vidéo source est déjà une vidéo compressée, vous pouvez descendre à 35-40-45.

Si la vidéo fait plus de 20MB par minute, il y a des chances que la vidéo soit mal convertie et que les paramètres de conversion soient trop élevés. Essayez de descendre la valeur de crf (25 dans notre exemple).
Il est possible que la vidéo fasse plus de 20MB par minute, surtout dans les cas où les plans changent beaucoup, ou bien beaucoup de flashs apparaissent, ou bien des effets de neige apparaissent dans la vidéo originale, etc.

`650KB` correspond au bitrate maximum de la vidéo à une seconde donnée. Si pour garder une bonne qualité, une vidéo doit utiliser plus d'espace que cette valeur par seconde, alors la qualité de la vidéo pendant ce temps à sera ajustée à 650KB (dans notre exemple). Cette valeur est une bonne valeur basique, mais si la vidéo possède de gros artefacts pendant les scènes de mouvement, il peut être bien d'augmenter voire de supprimer cette valeur (ce qui veut dire aucune limite).

### Convertir une vidéo sous Media Codec

à venir

## Fichier de sous-titres

Cela va de soit, mais le fichier json doit être un fichier json valide. Si vous avez un doute, utilisez un des nombreux site qui permet de valider un fichier json en ligne, par exemple [jsonlint.com](jsonlint.com).

Si le fichier json est invalide, vous aurez une erreur dans les fichiers de log et directement sur l'écran en mode édition ou en mode normal.

Voici un exemple de [fichier json pour l'opening 1 de JoJo no Kimyou na Bouken](http://0bin.net/paste/QTWN2gz-EZX5P6xS#XDta7i5YRPBP7uZ7GTypyIoyIwdLZEYqGoqaLx951nk).

De base, la structure d'un fichier JSON pour sous titres est comme indiqué ci dessous (les annotations comme @Syllabe référencent vers une autre structure expliquée plus tard). Le fichier ci dessus sert juste d'exemple, ce n'est en aucun cas un fichier valide !!

```hjson
{
  "sentences": [
    {
      "syllables": [
        @Syllable,
        @Syllable,
        @Syllable,
        ...
      ],
      "sentence_options":@SentenceOptions
    },
    {
      "syllables":[
        @Syllable,
        @Syllable
      ],
      "sentence_options":@SentenceOptions
    }
  ],
  "subtitles_options": {
    "sentence_options":@SentenceOptions,
    "start_credits_time": 4000, // temps en ms du début des premiers crédit
    "end_credits_time": 78010, // temps en ms du début des seconds crédits
    "credits_time": 8000 // durée totale des crédits
  },
  "song_info": { // optionnel, doit être rempli automatiquement par un script depuis le yaml
    "artist": "Hiroaki TOMMY Tominaga",
    "media_title": "Jojo no Kimyou na Bouken",
    "song_name": "JoJo ~Sono Chi no Sadame~",
    "music_type": "OP",
    "music_number": 1,
    "media_type": "Anime",
    "language": "JAP",
    "year": 2012
  }
}
```
### SentenceOptions

toutes les valeurs de SentenceOptions sont optionnelles; si aucune valeur n'est définie, une valeur par défaut sera prise en compte.
La plupart du temps, la valeur par défaut convient.
```hjson
{
    "syllable_options": @SyllableOptions, // ces options de syllabe sont appliqués sur chaque syllabe de la phrase en question
    "transitions": [
      @Transition,
      @Transition,
      ...
    ],
    "display_logo": true, // true | false : affiche ou non le logo sous la syllabe en cours 
    "transition_time_before": 600, // temps en ms
    "fade_time_before": 200, // temps en ms
    "transition_time_after": 300, // temps en ms
    "fade_time_after": 200, // temps en ms
    "row_position": @RowPosition,
    "size": @Size
}
```

### Syllable

```hjson
{
  "text":"da",// texte de la syllabe
  "begin":500,// temps en ms du début de la syllabe
  "end":1500, // temps en ms de la fin de la syllabe
  "syllable_options": @SyllableOptions // Options pour la syllabe en particulier; valeur optionelle.
}
```

### SyllableOptions

toutes ls valeurs de SyllableOptions sont optionnelles; si aucune valeur n'est définie, une valeur par défaut sera prise à la place.

```hjson
{
    "alive_color" : @Color, // Couleur des syllabes avant qu'elles soient chantées
    "transition_color" : @Color, // Couleur de la syllabe pendant qu'elle est chantée
    "dead_color" : @Color, // Couleur de la syllabe après qu'elle soit chantée
    "outline" : { // définit la bordure
      "size":1 // 0 : pas de bordure, 1 : bordure standard, 2 bordure large,
      "color":@Color
    }
}
```

### RowPosition

De base la position d'une syllabe est calculé pour que 2 phrases ne se coupent pas entre elles. Il peut être cependant pratique de mettre une phrase en particulier en bas de l'écran; RowPosition permet une telle possibilité.

RowPosition peut être définit de 2 façons :

* soit en spécifiant la ligne directement, en forçant la syllabe à être à la ligne donnée (et en évitant que les autres phrases s'entrechoquent).
* soit en spécifiant la position en x et en y directement.

Cela donne ceci :

```hjson
   ...
   "position":5 // met la phrase à la ligne 5, càd la dernière ligne avant coupure du logo.
   ...
```

ou cela : 

```hjson
   "position":{
       "x":0.5,
       "y":0.75
   }
```

à noter qu'aucun calcul de collision ne sera fait pour cette seconde possibilité.
Cette dernière possibilité centre le centre de la phrase en x. Une valeur "x" = 0.25 aurait mis le centre de la phrase à 25% de la position en x.
Pour y, le point le plus haut est placé à 75% de la position de l'écran. y = 0

### Size

Size devrait plutot s'appeler "FitSize". Elle se présente comme suit : 

```hson
"size":{
    "width":0.9, // optionnel
    "height":0.2 // optionnel
}
```

Les deux valeurs sont optionnelles mutuellement; si l'une manque, l'autre n'est pas optionelle !

"width" et "height" correspond tous les deux aux pourcentages de la taille de l'écran (où 1 équivaut à 100%) auquel le texte (ou la phrase) donnée doit rentrer dans les dimensions mentionnées. Dans l'exemple ci dessus, le texte ne pourra pas dépasser 90% de la taille de la fenêtre en largeur et 20% de la taille de la fenêtre en hauteur. Les textes dits "normaux" sont environ à 8% de la taille en hauteur, et 95% de la taille en largeur. Si vous voulez faire une texte deux fois plus grand qu'une phrase normale, `"height":0.18` devrait suffire.

Si une des valeurs n'est pas donnée, cela veut dire que la paramètre manquant est considéré comme illimité. Dans notre exemple, si la prhase est trop longue et dépasse initialement 90%, le texte deviendra plus petit. Enlever `"width":0.9` fera en sorte que le texte garde la même taille quelque soit la longueur de la phrase, mais il est possible que la phrase ne rentre pas dans l'écran !

Si les deux paramètres sont manquants, une erreur de parsing est renvoyée. En effet, techniquement le texte aurait une taille illimitée ...

### Color

```hjson
{
  "red":255,
  "blue":255,
  "green":255
}
```

On peut aussi tout simplement remplacer "Color" par une valeur hexadécimale RGB comme "#RRGGBB", par exemple, 

### Transition

Une transition change certaines options de toute la phrase instantanément à un instant donné. Offset correspond à la valeur de la frame comparé à la valeur "begin" de la première syllabe de la phrase. Cette valeur peut donc être négative si une transition doit apparaitre avant que la première syllabe ne soit chantée.

```hjson
{
"offset":-500,
"new_options":@SentenceOptions
}
```

Plusieurs transitions sont possibles par phrase, mais une transition ne touche que la phrase dans laquelle elle est. Par exemple, le violet serait la couleur hexadécimale "#551A8B" et le rouge brut serait la couleur hexadécimale "FF0000"

## Fichier de metadonnées

Le fichier de métadonnées est défini par un fichier yaml. Le fichier yaml et le fichier json possèdent tous deux des informations sur la musique, mais les informations dans le json ne servent qu'à la génération des crédits uniquement !
Le fichier yaml, lui, est utiisé pour la recherche, le tri, etc.

Un fichier yaml est défini comme tel :

```yaml
video_path: "JoJo no Kimyou na Bouken - OP1 - JoJo ~Sono Chi no Sadame~.webm"
json_path: "JoJo no Kimyou na Bouken - OP1 - JoJo ~Sono Chi no Sadame~.json" # Valeur optionelle; il peut être pratique d'avoir plusieurs vidéos pointer sur un seul json
song_info:
        media_title: "Jojo no Kimyou na Bouken" # le nom de la série
        media_alt_titles: # Liste des noms alternatifs pour la recherche (nom anglais, nom français, ...)
            - "Jojo's bizarre adventures"
        music_number: 1 # le numéro de la musique (ie OP"1")
        music_type: Opening # Opening | Ending | Insert | OST | AMV
        song_name: "JoJo ~Sono Chi no Sadame~" # Le nom de la musique elle même
        year: 2012 # Année de parution de l'opening/ending/...
        language: "Jp" # Langue de la musique : JAP | FR | ENG | RUS | GER | INSTRUMENTAL | ...
        media_type: "Anime" # Anime | Movie | VideoGame | Original (eg Touhou, Vocaloid, ...)
        artist: "Hiroaki TOMMY Tominaga" # nom du chanteur
time_info:
        timer: "Votre nom" # nom du timer
        license: "Anyone" # Qui peut utiliser ce time ?
        creditless: false # true | false ; est ce que des crédits sont présents dans la video ?
```

Toutes les valeurs de time_info et song_info sont optionnelles. Pour song_info, remplir ces valeurs permet d'avoir une recherche plus précise (et d'avoir la génération des crédits dans le fichier json). Cela permet à terme d'avoir une recherche par langue, par auteur, ...


# Annexe

## A propos de webm

Le format webm ne possède que des formats libres de droits et utilisables sans modération
(contrairement au conteneur mp4 qui peut avoir certains formats payants au bout d'une certaine quantité, tel que hevc ou h264).

Webm est voué à devenir présent partout sur le web dans les années à venir, et donc le support ne manquera absolument pas.
