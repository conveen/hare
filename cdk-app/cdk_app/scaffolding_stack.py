## Copyright (c) 2020 conveen
##
## Permission is hereby granted, free of charge, to any person obtaining a copy
## of this software and associated documentation files (the "Software"), to deal
## in the Software without restriction, including without limitation the rights
## to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
## copies of the Software, and to permit persons to whom the Software is
## furnished to do so, subject to the following conditions:
##
## The above copyright notice and this permission notice shall be included in all
## copies or substantial portions of the Software.
##
## THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
## IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
## FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
## AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
## LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
## OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
## SOFTWARE.

from typing import Optional

from aws_cdk import (
    aws_certificatemanager as acm,
    aws_ec2 as ec2,
    aws_elasticloadbalancingv2 as elbv2,
    aws_iam as iam,
    core,
)


__author__ = "conveen"
_INSTANCE_CLASS_T3_NANO = ec2.InstanceType.of(
    ec2.InstanceClass.BURSTABLE3,
    ec2.InstanceSize.NANO
)


class BastionHostWithRole(core.Construct):
    """Amazon Linux bastion host with IAM role to connect
    with SSH via EC2 Connect and SSM Session Manager.
    """

    def __init__(self,
                 scope: core.Stack,
                 id: str,
                 vpc: ec2.Vpc,
                 subnet: ec2.SubnetSelection,
                 instance_name: Optional[str] = None,
                 instance_type: Optional[ec2.InstanceType] = _INSTANCE_CLASS_T3_NANO):
        super().__init__(scope, id)

        # NOTE: Users must have the proper policy on their IAM user/role to connect to the instance via SSM.
        #       See: https://docs.aws.amazon.com/systems-manager/latest/userguide/getting-started-restrict-access-quickstart.html
        self.host = ec2.BastionHostLinux(self, "BastionHost",
                                         vpc=vpc,
                                         instance_name=instance_name,
                                         instance_type=instance_type,
                                         subnet_selection=subnet)
        # See https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-connect-set-up.html#ec2-instance-connect-configure-IAM-role
        # and https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-getting-started-enable-ssh-connections.html
        host_arn = f"arn:aws:ec2:{scope.region}:{scope.account}:instance/{self.host.instance_id}"
        ec2_describe_instances_policy = iam.PolicyStatement(actions=["ec2:DescribeInstances"], resources=["*"])
        ec2_connect_send_ssh_policy = iam.PolicyStatement(
            actions=["ec2-instance-connect:SendSSHPublicKey"],
            resources=[host_arn],
            conditions={"StringEquals": {"ec2:osuser": "ssm-user"}}
        )
        ssm_start_session_policy = iam.PolicyStatement(
            actions=["ssm:StartSession"],
            resources=[
                host_arn,
                "arn:aws:ssm:*:*:document/AWS-StartSSHSession"
            ]
        )
        # Have to set a trust policy with all users in the stack account, _and_ allow individual users
        # to AssumeRole for this specific role. Part 1 is accomplished here, part 2 is manually done later
        # on a per-user basis.
        self.connect_role = iam.Role(self, "BastionHostSSHConnectRole",
                                     assumed_by=iam.AccountPrincipal(scope.account),
                                     description="Connect to bastion host via SSH using SSM Session Manager",
                                     inline_policies={"SSHConnect": iam.PolicyDocument(statements=[
                                         ec2_describe_instances_policy,
                                         ec2_connect_send_ssh_policy,
                                         ssm_start_session_policy,
                                     ])})


class Network(core.Construct):
    """VPC and other networking configurations."""

    def __init__(self, scope: core.Construct, id: str):
        super().__init__(scope, id)
        ingress_subnet = ec2.SubnetConfiguration(name="Ingress",
                                                 subnet_type=ec2.SubnetType.PUBLIC,
                                                 cidr_mask=24)
        application_subnet = ec2.SubnetConfiguration(name="Application",
                                                     subnet_type=ec2.SubnetType.ISOLATED,
                                                     cidr_mask=24)
        self.vpc = ec2.Vpc(self, "VPC",
                           cidr="172.22.0.0/16",
                           max_azs=1,
                           nat_gateways=0,
                           subnet_configuration=[ingress_subnet, application_subnet])
        self.subnets = {
            "IngressSelection": ec2.SubnetSelection(
                subnets=self.vpc.select_subnets(subnet_type=ec2.SubnetType.PUBLIC).subnets
            ),
            "ApplicationSelection": ec2.SubnetSelection(
                subnets=self.vpc.select_subnets(subnet_type=ec2.SubnetType.ISOLATED).subnets
            ),
        }
        # We are using NACLs to accomplish the following:
        #   1) Allow all outbound traffic
        #   2) Allow all internal traffic
        #   3) Allow inbound traffic from any IP to ports 80 and 443
        ingress_subnet_nacl = ec2.NetworkAcl(self, "IngressNacl",
                                             vpc=self.vpc,
                                             subnet_selection=self.subnets["IngressSelection"])
        ingress_subnet_nacl.add_entry("AllowAllEgressEntry",
                                      cidr=ec2.AclCidr.any_ipv4(),
                                      rule_number=10,
                                      traffic=ec2.AclTraffic.all_traffic(),
                                      direction=ec2.TrafficDirection.EGRESS,
                                      rule_action=ec2.Action.ALLOW)
        ingress_subnet_nacl.add_entry("AllowAllInternalIngressEntry",
                                      cidr=ec2.AclCidr.ipv4(self.vpc.vpc_cidr_block),
                                      rule_number=10,
                                      traffic=ec2.AclTraffic.all_traffic(),
                                      direction=ec2.TrafficDirection.INGRESS,
                                      rule_action=ec2.Action.ALLOW)
        for entry_id, port_number, rule_number in (
            ("AllowHttpIngressEntry", 80, 20),
            ("AllowHttpSIngressEntry", 443, 30),
        ):
            ingress_subnet_nacl.add_entry(entry_id,
                                          cidr=ec2.AclCidr.any_ipv4(),
                                          rule_number=rule_number,
                                          traffic=ec2.AclTraffic.tcp_port(port_number),
                                          direction=ec2.TrafficDirection.INGRESS,
                                          rule_action=ec2.Action.ALLOW)


class LoadBalancer(core.Construct):
    """Load balancer and associated target groups."""

    def __init__(self, scope: core.Construct, id: str, network: Network, certificate_arn: str):
        super().__init__(scope, id)

        self.load_balancer = elbv2.ApplicationLoadBalancer(self, "LoadBalancer",
                                                           vpc=network.vpc,
                                                           internet_facing=True,
                                                           load_balancer_name="hare-elb-01",
                                                           vpc_subnets=network.subnets["IngressSelection"])
        certificate = acm.Certificate.from_certificate_arn(self, "LoadBalancerCertificate", certificate_arn)
        # NOTE: Path patterns can include wildcards, so only need one set of listeners.  Routing will be done by paths in request.
        #       See: https://docs.aws.amazon.com/elasticloadbalancing/latest/application/load-balancer-listeners.html#path-conditions
        self.listeners = {
            "Https": self.load_balancer.add_listener("HttpsListener",
                                                     open=True,
                                                     port=443),
            "Http": self.load_balancer.add_listener("HttpListener",
                                                    open=True,
                                                    port=80)
        }
        self.listeners["Https"].add_certificates("LoadBalancerListenerCertificate", certificates=[
            elbv2.ListenerCertificate(certificate.certificate_arn)
        ])
        self.listeners["Http"].add_action("RedirectHttpToHttps", action=elbv2.ListenerAction.redirect(port="443"))

        self.target_group = elbv2.ApplicationTargetGroup(self, "WebTargetGroup",
                                                         port=80,
                                                         target_group_name="hare-engine",
                                                         target_type=elbv2.TargetType.IP,
                                                         vpc=network.vpc)
        (self
            .listeners["Https"]
            .add_target_groups("Web",
                               target_groups=[self.target_group]))


class ScaffoldingStack(core.Stack):
    """Network and scaffolding stack."""

    def __init__(self, scope: core.App, id: str, certificate_arn: str, **kwargs):
        super().__init__(scope, id, **kwargs)

        self.network = Network(self, "Network")

        self.bastion = BastionHostWithRole(self, "BastionHostWithRole",
                                           vpc=self.network.vpc,
                                           subnet=self.network.subnets["IngressSelection"],
                                           instance_name="hare-bastion-01")

        self.load_balancer = LoadBalancer(self, "LoadBalancer", network=self.network, certificate_arn=certificate_arn)
