# "Moka machine" proof-of-concept

## An off-the-shelf resonant driver

<small>2022-09-27 Tuesday (or later)</small>

For a proof-of-concept prototype I've got an off-the-shelf resonant driver from AliExpress ([lot](https://www.aliexpress.com/item/33010919113.html)).

Below are the oscillograms of the gate and drain voltages of the power MOSFETs of the driver with no load (no big difference under load).

![Resonant driver gate voltages](images/001%20ZVS%20gate%20voltage.png) ![Resonant driver drain voltages](images/002%20ZVS%20drain%20voltage.png)

This driver is well described on the page ["1000 Watt ZVS Induction Heater Notes"](https://spaco.org/Blacksmithing/ZVSInductionHeater/1000WattZVSInductionHeaterNotes.htm).

## Resonant driver control interface

<small>2022-10-01 Saturday, 2022-10-02 Sunday</small>

The old technique of scratching out a circuit board combined with SMD components.

![Control attachment back](images/003%20IMG_4103.jpeg) ![Control attachment front](images/004%20IMG_4104.jpeg)

![Control attachment installed](images/005%20IMG_4105.jpeg) ![Control attachment connector](images/006%20IMG_4106.jpeg)

And the new coil, unfinished.

![The new coil, unfinished](images/007%20IMG_4107.jpeg)

## The new coil

<small>2022-10-03 Monday</small>

An attempt to boil a tiny amount of water with the coil.

![The new coil test](images/008%20IMG_4108.jpeg)

During the test I've realized that the coil wire near the mounting points is too loose and may create an undesired loop with extra inductance. Before closing the lab for the night, I had mounted the coil in a better way.

![A better way to mount the coil](images/009%20IMG_4109.jpeg)

## A super rough estimation of power at higher supply voltages

Shouldn't have even attempted it.

![A super rough estimation of power at higher supply voltages](images/010%20IMG_4112.jpeg)

On the other hand, if this is even remotely in the right ballpark, it supports my original idea of using a 24 V 400 W power supply for the PoC.
