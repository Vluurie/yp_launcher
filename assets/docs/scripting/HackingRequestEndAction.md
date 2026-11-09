# HackingRequestEndAction

Ends the hacking minigame. It fires once and is done straight away.

## endType

| endType | What happens |
|---|---|
| 0 | the hack just ends |
| 1 | the player is destroyed and set down again at the retry point |

With 1 and no retry point set up, the hack ends anyway.

## useSelect

| useSelect | What happens |
|---|---|
| 0 | no menu at the end |
| 1 | the player gets the menu |

This one is on unless you write 0. Every other field here is off unless you write 1.

## endSkip

| endSkip | What happens |
|---|---|
| 0 | you watch the way out: effect, sound, the camera swinging back |
| 1 | the screen fades and the game loads instead |

## ForceSkip

| ForceSkip | What happens |
|---|---|
| 0 | the result screen is shown first |
| 1 | no result screen, and the fade-and-load way out |

## endCamPos and endCamTag

| Filled in | What happens |
|---|---|
| neither | the camera is left alone |
| both | the camera moves to `endCamPos` and looks at `endCamTag` |

Filling in only one of the two does nothing.

## FadeBlack and FadeWhite

| Value | What happens |
|---|---|
| 0 | no fade |
| 1 | the screen fades, to black or to white |

## A real one from the game

```xml
<action>
	<code str="HackingRequestEndAction">0x2662e9b7</code>
	<name eng="termination request">終了リクエスト</name>
	<id>0xda9a2a7f</id>
	<attribute>0x2</attribute>
	<useSelect>0</useSelect>
	<endSkip>1</endSkip>
	<ForceSkip>1</ForceSkip>
</action>
```

Left out here: `FadeWhite`, `endCamPos`, `endCamTag` and `endType`. They are optional, and leaving them out keeps the defaults.
