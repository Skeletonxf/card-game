import React from "react"

import Greeting from "./components/Greeting"
import NameChange from "./components/NameChange"
import Cell from "./components/Cell"

import "./App.css";

let Coordinate: [number, number]

const grid: Coordinate[][] = [
    [[0, 2], [0, 3], [0, 4]],
    [[1, 0], [1, 1], [1, 2], [1, 3], [1, 4], [1, 5], [1, 6]],
    [[2, 0], [2, 1], [2, 2], [2, 3], [2, 4], [2, 5], [2, 6]],
    [[3, 0], [3, 1], [3, 2], [3, 3], [3, 4], [3, 5], [3, 6]],
    [[4, 0], [4, 1], [4, 2], [4, 3], [4, 4], [4, 5], [4, 6]],
    [[5, 2], [5, 3], [5, 4]],
]

const App = () => {
    return (
        <div className="App">
            <div className="flex flex-col">
                <Greeting />
                <NameChange />
            </div>
            <div
                className="grid"
                style={{
                    gridTemplateColumns: `repeat(7, fit-content(${100.0/7.0}%))`
                }}
            >
                {
                    grid.map((row: Coordinate[]) => {
                        return (
                            row.map(([x, y]: Coordinate) => {
                                return (
                                    <div
                                        key={`${x}-${y}`}
                                        style={{
                                            gridArea: `${x+1} / ${y+1} / ${x+1} / ${y+1}`
                                        }}
                                    >
                                        <Cell x={x} y={y} />
                                    </div>
                                )
                            })
                        )
                    })
                }
            </div>
        </div>
    );
};

export default App;
