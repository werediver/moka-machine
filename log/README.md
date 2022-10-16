# "Moka machine" proof-of-concept

## An off-the-shelf resonant driver

_2022-09-27 Tuesday (or a few days later)_

For a proof-of-concept prototype I've got an off-the-shelf resonant driver from AliExpress ([lot](https://www.aliexpress.com/item/33010919113.html)).

Below are the oscillograms of the gate and drain voltages of the power MOSFETs of the driver with no load (no big difference under load) with 12 V supply.

<p width="100%">
<img width="49%" alt="Resonant driver gate voltages" src="images/001%20Resonant driver%20gate%20voltage.png">
<img width="49%" alt="Resonant driver drain voltages" src="images/002%20Resonant driver%20drain%20voltage.png">
</p>

This driver is well described on the page ["1000 Watt ZVS Induction Heater Notes"](https://spaco.org/Blacksmithing/ZVSInductionHeater/1000WattZVSInductionHeaterNotes.htm).

## Resonant driver control interface

_2022-10-01 Saturday, 2022-10-02 Sunday_

The old technique of scratching out a circuit board combined with SMD components.

<p width="100%">
<img width="49%" alt="Control attachment back" src="images/003%20IMG_4103.jpeg">
<img width="49%" alt="Control attachment front" src="images/004%20IMG_4104.jpeg">
</p>

<p width="100%">
<img width="49%" alt="Control attachment installed" src="images/005%20IMG_4105.jpeg">
<img width="49%" alt="Control attachment connector" src="images/006%20IMG_4106.jpeg">
</p>

And the new coil made of 4x1 mm² "audio" cable, unfinished. The cross-section of the pipe of the original coil is around 10 mm².

![The new coil, unfinished](images/007%20IMG_4107.jpeg)

## The new coil

_2022-10-03 Monday_

An attempt to boil a tiny amount of water with the coil. With the supply voltage of 15 V the current consumption is 4.47 A, about 67 W. The water heated up to 50℃.

![The new coil test](images/008%20IMG_4108.jpeg)

During the test I've realized that the coil wire near the mounting points is too loose and may create an undesired loop with extra inductance. Before closing the lab for the night, I had mounted the coil in a better way.

![A better way to mount the coil](images/009%20IMG_4109.jpeg)

## A super rough estimation of power at higher supply voltages

_2022-10-04 Tuesday_

<details>
<summary>
Shouldn't have even attempted it
</summary>

![A super rough estimation of power at higher supply voltages](images/010%20IMG_4112.jpeg)

</details>

Difficult to make a prediction, but my slightly educated guess is that at 24 V a 400 W power supply should be sufficient for this contraption.

## Ways to achieve higher power

_2022-10-05 Wednesday_

Those 67 W with 15 V x 4.47 A input is close to the current limit of my lab PSU, so I wouldn't be able to just raise the voltage to achieve (much) higher output power. I have two of PSUs, though. With some further modification of the driver circuit, a split PSU may be used to power it, thus doubling the output power.

![Resonant driver Split PSU](images/011%20Resonant%20driver%20split%20PSU.png)

Or I could use some balancing resistors and just connect the two PSUs in parallel.

## Closed loop control

_2022-10-08 Saturday_

I implemented some basic (bang-bang with a deadband ±0.2℃ as per the NCIR thermometer) closed loop control logic in the firmware and ran a test.

At first, the program was stopping when the resonant converter was turning on or off. Trying to fight this I put ferrite rings on the longest wires coming to the NCIR thermometer and the resonant converter.

![Pico attached to the NCIR thermometer and the resonant converter](images/012%20IMG_4119.jpeg)

That didn't have sufficient effect, so I had to use stronger countermeasures...

![A test set-up with closed loop control](images/013%20IMG_4117.jpeg)

After which some stability was achieved and I could keep the target (a steel steaming pitcher) at a set temperature.

Maybe not very spectacular, but here is [a short video](https://odysee.com/@werediver:d/moka-machine-01:7?r=EgVnnPDpYAySnwJ9STYyvCuVqFXdCxUz).

Notice that I'm using the oscilloscope ground lead as a makeshift EMI probe. Learned the trick in [this video](https://youtu.be/WytDROmjWKQ?t=129).

## It was ~~DNS~~ the debug probe

_2022-10-09 Sunday_

After some more experimentation and debugging I made two conclusions:

- the issues on the heater start/stop were not with the controller, but rather with the debug probe: even though RTT debug output was stopping, the controller was still running (confirmed with oscilloscope-assisted debugging)
- the NCIR thermometer really likes black and/or high IR emissivity objects or it gives underestimated (possibly, influenced by the reflected surroundings) readings and setting a lower object emissivity parameter seems to give somewhat unstable results (quite sensitive to the sensor position)

Under that particular load and power supply parameters (~13.8 V x 5 A) the resonant converter starts oscillating 1.6 ms after activation and reaches full amplitude in about 30 ms.
