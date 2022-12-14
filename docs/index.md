---
title: Direccionamiento de Puertos dinamico con eBPF en servidores Linux
authors: ["Christopher Valerio", "Gabriel Granados", "Jose Pablo Porras", "Miguel Soto"]
date: 2022-11-14
keywords: ["linux", "networking"]
bibliography: bibliography.bib
toc-own-page: true
header-includes:
- |
  ```{=latex}
  \usepackage{awesomebox}
  ```
pandoc-latex-environment:
  noteblock: [note]
  tipblock: [tip]
  warningblock: [warning]
  cautionblock: [caution]
  importantblock: [important]

  
abstract: | 
    Las Redes donde se despliegan sistemas  IoT muchas veces no cuentan
    con las mismas condiciones que un datacenter o servicios en la nube,
    por lo cual es dificil tener las mismas configuraciones y usar los mismos
    metodos que se usan para enrutar trafico que en estas. Un ejemplo son `Software Defined Networks`
    que permiten dinamicamente a los sistemas configurar sus sistemas de enrutamientos. El presente articulo
    describe las tecnologias disponibles asi como sus posibles aplicaciones en 
    un caso de uso como el descrito, que ventajas trae el lenguaje de programacion Rust
    o protocolos como grpc, asi mismo la potencia de nuevas tecnologias integradas al kernel de linux
    como lo es eBPF.
---

## Tecnologias

### eBPF

eBPF es una maquina virtual que corre en kernel de linux que permite ejecutar aplicaciones 
a nivel del espacio de kernel haciendo que sean muy eficientes y rapida[@8850758]


::: note
Lorem ipsum dolor ...
:::

### Rust

Lenguaje de programación de alto nivel que permite desarrollar aplicaciones que aprovechan al máximo el hardware
al ser capaz de acceder a los elementos que este provee similar a lenguajes como `C` o `C++` así como 

### gRPC


## Arquitectura de Red 


## Arquitectura de Software


## Conclusiones

Es un protocol binario para crear sistemas remotos 

## Bibliografía
