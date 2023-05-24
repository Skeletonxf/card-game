import React from "react";

type Props = {
    x: number;
    y: number;
}

const Cell: React.FC<Props> = ({ x, y }) => {
    return (
        <div className="cell">
            {`(${x}, ${y})`}
        </div>
    )
}

export default Cell;
