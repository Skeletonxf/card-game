import React, { useMemo, useState } from "react";

import { setUserName } from "../store/user/userSlice";
import { selectUserName } from "../store/user/selectors";
import { useAppDispatch, useAppSelector } from "../hooks/stateHooks";
import Button from "./Button";

/**
 * An input that allows the user to change their name.
 */
const NameChange = () => {
    const dispatch = useAppDispatch();

    /** The name of the user. */
    const userName = useAppSelector(selectUserName);

    /** The name of the user in the input, defaulting to the user name. */
    const [inputUserName, setInputUserName] = useState<string>(userName);

    /**
     * Indicates whether the user can use the "Submit" button to change their
     * name or not. This is only true if the name in the input is different
     * than the name in the state.
     */
    const canChangeName = useMemo(
        () => userName !== inputUserName,
        [userName, inputUserName],
    );

    /**
     * Called when the name of the user is updated in the input to update the
     * name in the local state.
     *
     * @param {React.ChangeEvent<HTMLInputElement>} event The change event.
     */
    const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) =>
        setInputUserName(event.target.value);

    /**
     * Called when the "Submit" button is pressed to set the new name for the
     * user in the store.
     */
    const handleNameChange = () => dispatch(setUserName(inputUserName));

    return (
        <div className="flex-column">
            <p>Don&apos;t like the name {userName}? Change It!</p>
            <div className="flex-row">
                <input
                    onChange={handleInputChange}
                    value={inputUserName}
                    className="text-field"
                />
                <button
                    onClick={handleNameChange}
                    disabled={!canChangeName}
                    className="button"
                >
                    Submit
                </button>
                <Button text="testing" onClick={() => alert('hello world')} />
            </div>
        </div>
    );
};

export default NameChange;
