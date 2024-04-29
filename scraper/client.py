import grpc
import score_pb2
import score_pb2_grpc
import joblib
import pandas as pd
import requests
import json
import numpy as np
import warnings

warnings.filterwarnings("ignore")
import pickle
from sklearn.preprocessing import OrdinalEncoder
import csv

pipeline = joblib.load("pipeline.joblib")
with open("pipeline.pkl", "rb") as f:
    pkl_pipeline = pickle.load(f)
# Find the OrdinalEncoder step in the pipeline


def convert_over_ball_to_ball_number(over_ball):
    """
    Converts a float in the format 'over.ball' to a ball number.
    Example: 0.1 -> 1, 5.6 -> 36
    """
    if pd.isna(over_ball):
        return None
    over = int(over_ball)
    ball = int((over_ball - over) * 10)
    return over * 6 + ball


team_mapping = {
    "Mumbai Indians": "MI",
    "Royal Challengers Bangalore": "RCB",
    "Kolkata Knight Riders": "KKR",
    "Chennai Super Kings": "CSK",
    "Rajasthan Royals": "RR",
    "Kings XI Punjab": "KXIP",
    "Sunrisers Hyderabad": "SRH",
    "Delhi Daredevils": "DD",
    "Delhi Capitals": "DC",
    "Deccan Chargers": "DC",
    "Pune Warriors": "PW",
    "Punjab Kings": "PBKS",
    "Gujarat Titans": "GT",
    "Lucknow Super Giants": "LSG",
    "Gujarat Lions": "GL",
    "Rising Pune Supergiant": "RPS",
    "Kochi Tuskers Kerala": "KTK",
    "Rising Pune Supergiants": "RPS",
    "Royal Challengers Bengaluru": "RCB",
}


def run():
    with grpc.insecure_channel("localhost:50051") as channel:
        stub = score_pb2_grpc.ScoreStreamStub(channel)
        request = score_pb2.StreamScoreRequest(match_id="match123")
        response_iterator = stub.StreamScore(request)
        for res in response_iterator:
            csv_file = open("prediction_history.csv", "a", newline="")
            csv_writer = csv.writer(csv_file)
            print("New Response: ")
            print(res)
            score = int(res.score)
            batsman = res.batsman.replace('"', "")
            currentRunRate = res.currentrunrate.replace('"', "")
            battingTeam = res.battingteam.replace('"', "")
            bowlingTeam = res.bowlingteam.replace('"', "")
            innings = int(res.innings)
            wickets = res.wickets.replace('"', "")
            overs = float(res.overs)
            tossWinner = res.tosswinner.replace('"', "")
            tossDecision = res.tossdecision.replace('"', "")
            team1 = res.team1.replace('"', "")
            team2 = res.team2.replace('"', "")
            umpire1 = res.umpire1.replace('"', "")
            umpire2 = res.umpire2.replace('"', "")
            umpire3 = res.umpire3.replace('"', "")
            venue = res.venue.replace('"', "").replace(",&nbsp;", "")
            matchId = res.matchid.replace('"', "")
            nonStriker = res.nonStriker.replace('"', "")
            bowler = res.bowler.replace('"', "")
            ball_number = convert_over_ball_to_ball_number(overs)
            if tossDecision == "Batting":
                tossDecision = "bat"
            if tossDecision == "Bowling":
                tossDecision = "field"
            input_feature_names = pkl_pipeline.named_steps[
                "cat_null_handler"
            ].feature_names_in_
            umpire1 = "Navdeep Singh"
            umpire3 = "Navdeep Singh"
            if batsman == "Tim David":
                batsman = "Travis Head"
            if nonStriker == "Tim David":
                nonStriker = "Travis Head"
            venue = "M Chinnaswamy Stadium"
            print(input_feature_names)
            input = [
                [
                    venue,
                    battingTeam,
                    bowlingTeam,
                    batsman,
                    nonStriker,
                    bowler,
                    team1,
                    team2,
                    team_mapping[tossWinner.replace('"', "")],
                    tossDecision,
                    umpire1,
                    umpire2,
                    umpire3,
                ]
            ]
            skip_prediction = False
            for element in input[0]:
                if element == "":
                    print("Found some null value so not gonna predict")
                    break
            if skip_prediction:
                print("quitting")
                continue
            print("The input for preprocessor: ", input)
            DEFAULT_VALUE = "Virat Kohli"
            try:
                transformed_input = pkl_pipeline.transform(input)
            except ValueError as e:
                if "Found unknown categories" in str(e):
                    # Handle the unknown category error
                    print(f"Encountered unknown category: {e}")
                    category_index = int(
                        str(e).split(" in column ")[1].split(" during")[0]
                    )
                    print(f"The error occured at index: {category_index}")
                    input[0][category_index] = DEFAULT_VALUE
                    transformed_input = pkl_pipeline.transform(input)
                else:
                    raise e
            print(pkl_pipeline.transform(input))
            transformed_input = pkl_pipeline.transform(input)[0]

            # Create a new input list `input_tf` with transformed features and additional features
            input_tf = list(transformed_input) + [ball_number, score]
            input_tf.insert(1, innings)
            print(input_tf)
            data = np.array(input_tf, dtype=np.float32)

            # Reshape the data to a 2D tensor (assuming your model expects a single instance)
            data = data.reshape(1, -1)  # (1, 12) for a single instance with 12 features

            # Prepare the request payload
            payload = {"instances": data.tolist()}

            # Send a prediction request to the TensorFlow Serving server
            url = "http://localhost:8605/v1/models/cricket_model:predict"
            headers = {"content-type": "application/json"}
            response = requests.post(url, data=json.dumps(payload), headers=headers)

            # Check if the request was successful
            if response.status_code == 200:
                # Get the prediction result

                prediction = response.json()["predictions"][0]
                print(f"Prediction: {prediction}")
                input_tf.append(prediction[0])
                csv_writer.writerow(input_tf)
                print(f"wrote row: {input_tf}")
            else:
                print(f"Error: {response.status_code} - {response.text}")

            csv_file.close()


if __name__ == "__main__":
    run()
