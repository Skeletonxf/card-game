import React from "react";

type Props = {
    text: string;
    onClick: () => void;
}

const Button: React.FC<Props> = ({ text, onClick }) => {
    return (
        <button onClick={onClick} className="button">
            {text}
        </button>
    )
}

export default Button;
