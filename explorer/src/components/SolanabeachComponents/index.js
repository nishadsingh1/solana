import React, { Component, Fragment } from "react";
import io from "socket.io-client";
import NetworkStats from "./NetworkStats";
import { Cluster, useCluster } from "../../providers/cluster";

class LandingPageInner extends Component {
  constructor(props) {
    super(props);

    // Connect socket before component is mounted

    this.socket = io("https://api.solanabeach.io:8443/mainnet");

    this.socket.on("connect", () => this.requestData());

    this.socket.on("error", (err) => {
      console.log("error", err);
    });
  }

  requestData() {
    this.socket.emit("request_dashboardInfo");
    this.socket.emit("request_validatorInfo");
    this.socket.emit("request_performanceInfo");
  }

  componentWillUnmount() {
    // Disconnect socket when component is unmounted
    if (this.socket) {
      this.socket.disconnect();
    }
  }

  render() {
    return (
      <Fragment>
        <div className="hero-wrapper bg-composed-wrapper withOverflowingBackground">
          <div className="card-header">
            <div className="row align-items-center">
              <div className="col">
                <h4 className="card-header-title">Live Network Statistics</h4>
              </div>
            </div>
          </div>
          <NetworkStats socket={this.socket} location={this.props.location} />
        </div>
      </Fragment>
    );
  }
}

function LandingPage() {
  let cluster = useCluster();
  if (cluster.cluster !== Cluster.MainnetBeta) {
    return null;
  }
  return <LandingPageInner />;
}

export default LandingPage;